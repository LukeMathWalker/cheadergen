use rustdoc_ir::{ScalarPrimitive, Type};
use rustdoc_processor::GlobalItemId;
use rustdoc_processor::queries::Crate;

use crate::Collection;
use crate::diagnostic::DiagnosticSink;
use rustdoc_resolver::{TypeAliasResolution, resolve_type};
use rustdoc_types::{Item, ItemEnum};

/// A resolved constant item ready for C codegen as `#define NAME VALUE`.
pub struct ConstantItem {
    /// The Rust item name (used as the `#define` macro name).
    pub name: String,
    /// The evaluated value from rustdoc (emitted verbatim after `#define`).
    pub value: String,
    /// The global rustdoc item ID, used for doc comment lookup at codegen time.
    pub rustdoc_id: GlobalItemId,
}

/// Try to resolve a constant item into a [`ConstantItem`].
///
/// Returns `None` (with a warning diagnostic) if:
/// - The type does not resolve to a [`ScalarPrimitive`] (non-primitive constants are unsupported).
/// - The constant has no evaluated `value` from rustdoc.
pub fn resolve_constant(
    item: &Item,
    krate: &Crate,
    collection: &Collection,
    diagnostics: &mut DiagnosticSink,
) -> Option<ConstantItem> {
    let ItemEnum::Constant { type_, const_ } = &item.inner else {
        unreachable!("Expected a constant item");
    };

    let name = item.name.clone().unwrap_or_else(|| "<unnamed>".to_string());

    let resolved = match resolve_type(
        type_,
        &krate.core.package_id,
        collection,
        &Default::default(),
        TypeAliasResolution::ResolveThrough,
    ) {
        Ok(t) => t,
        Err(e) => {
            diagnostics
                .error(format!("constant `{name}`: failed to resolve type"))
                .with_span_if(item.span.as_ref())
                .with_error_chain(&e)
                .emit();
            return None;
        }
    };

    let value = match &resolved {
        // Bool: rustdoc emits "true"/"false", valid C as-is.
        Type::ScalarPrimitive(ScalarPrimitive::Bool) => {
            let Some(ref value) = const_.value else {
                diagnostics
                    .error(format!("constant `{name}` has no evaluated value"))
                    .with_span_if(item.span.as_ref())
                    .emit();
                return None;
            };
            value.clone()
        }
        // Char: rustdoc doesn't evaluate char literals, so use `expr`.
        // Only emit simple literals (not computed expressions).
        Type::ScalarPrimitive(ScalarPrimitive::Char) => {
            if !const_.is_literal {
                diagnostics
                    .error(format!("constant `{name}` is a computed char expression"))
                    .with_span_if(item.span.as_ref())
                    .with_help("only literal char constants are supported")
                    .emit();
                return None;
            }
            match sanitize_char_literal(&const_.expr) {
                Some(value) => value,
                None => {
                    diagnostics
                        .error(format!(
                            "constant `{name}` has a malformed char literal: `{}`",
                            const_.expr
                        ))
                        .with_span_if(item.span.as_ref())
                        .emit();
                    return None;
                }
            }
        }
        // Numeric types: strip Rust suffixes and underscores.
        Type::ScalarPrimitive(prim)
            if !matches!(
                prim,
                ScalarPrimitive::Str | ScalarPrimitive::U128 | ScalarPrimitive::I128
            ) =>
        {
            let Some(ref value) = const_.value else {
                diagnostics
                    .error(format!("constant `{name}` has no evaluated value"))
                    .with_span_if(item.span.as_ref())
                    .emit();
                return None;
            };
            sanitize_rust_number(value, prim)
        }
        // &str: rustdoc doesn't evaluate string literals, so use `expr`.
        // Only emit simple literals (not computed expressions).
        Type::Reference(r) if matches!(&*r.inner, Type::ScalarPrimitive(ScalarPrimitive::Str)) => {
            if !const_.is_literal {
                diagnostics
                    .error(format!("constant `{name}` is a computed string expression"))
                    .with_span_if(item.span.as_ref())
                    .with_help("only literal string constants are supported")
                    .emit();
                return None;
            }
            const_.expr.clone()
        }
        _ => {
            diagnostics
                .error(format!("constant `{name}` has unsupported type"))
                .with_span_if(item.span.as_ref())
                .emit();
            return None;
        }
    };

    Some(ConstantItem {
        name,
        value,
        rustdoc_id: GlobalItemId::new(item.id, krate.core.package_id.clone()),
    })
}

/// Try to resolve an associated constant into a [`ConstantItem`].
///
/// Unlike free-standing constants, `AssocConst` has no `expr` or `is_literal` fields —
/// only `value: Option<String>`. Rustdoc uses `"_"` as a placeholder when it cannot
/// evaluate the expression (e.g. const fn results), so those are skipped.
///
/// The emitted name is `{type_name}_{const_name}` (e.g. `Foo_GA`).
pub fn resolve_assoc_constant(
    item: &Item,
    type_name: &str,
    krate: &Crate,
    collection: &Collection,
    diagnostics: &mut DiagnosticSink,
) -> Option<ConstantItem> {
    let ItemEnum::AssocConst { type_, value } = &item.inner else {
        unreachable!("Expected an AssocConst item");
    };

    let const_name = item.name.clone().unwrap_or_else(|| "<unnamed>".to_string());

    let resolved = match resolve_type(
        type_,
        &krate.core.package_id,
        collection,
        &Default::default(),
        TypeAliasResolution::ResolveThrough,
    ) {
        Ok(t) => t,
        Err(e) => {
            diagnostics
                .error(format!(
                    "assoc constant `{type_name}_{const_name}`: failed to resolve type"
                ))
                .with_span_if(item.span.as_ref())
                .with_error_chain(&e)
                .emit();
            return None;
        }
    };

    let raw_value = match value {
        Some(v) if !v.is_empty() && v != "_" => v,
        _ => {
            diagnostics
                .error(format!(
                    "assoc constant `{type_name}_{const_name}` has no evaluated value"
                ))
                .with_span_if(item.span.as_ref())
                .emit();
            return None;
        }
    };

    let value = match &resolved {
        Type::ScalarPrimitive(ScalarPrimitive::Bool) => raw_value.clone(),
        Type::ScalarPrimitive(ScalarPrimitive::Char) => match sanitize_char_literal(raw_value) {
            Some(value) => value,
            None => {
                diagnostics
                        .error(format!(
                            "assoc constant `{type_name}_{const_name}` has a malformed char literal: `{raw_value}`"
                        ))
                        .with_span_if(item.span.as_ref())
                        .emit();
                return None;
            }
        },
        Type::ScalarPrimitive(prim)
            if !matches!(
                prim,
                ScalarPrimitive::Str | ScalarPrimitive::U128 | ScalarPrimitive::I128
            ) =>
        {
            sanitize_rust_number(raw_value, prim)
        }
        Type::Reference(r) if matches!(&*r.inner, Type::ScalarPrimitive(ScalarPrimitive::Str)) => {
            raw_value.clone()
        }
        _ => {
            diagnostics
                .error(format!(
                    "assoc constant `{type_name}_{const_name}` has unsupported type"
                ))
                .with_span_if(item.span.as_ref())
                .emit();
            return None;
        }
    };

    Some(ConstantItem {
        name: format!("{type_name}_{const_name}"),
        value,
        rustdoc_id: GlobalItemId::new(item.id, krate.core.package_id.clone()),
    })
}

/// Convert a Rust char literal (from rustdoc's `expr`) to a C-compatible form.
///
/// ASCII chars pass through unchanged, as do the escape sequences C shares
/// with Rust (`\n`, `\t`, `\\`, `\'`, `\0`, `\xNN`, …). Rust-only `\u{…}`
/// escapes and non-ASCII chars are converted to C11 universal character
/// names. Returns `None` if the literal is malformed.
fn sanitize_char_literal(expr: &str) -> Option<String> {
    // expr is like "'X'" — strip surrounding quotes
    let inner = expr.strip_prefix('\'')?.strip_suffix('\'')?;
    if let Some(escape) = inner.strip_prefix('\\') {
        if let Some(hex) = escape.strip_prefix("u{").and_then(|h| h.strip_suffix('}')) {
            // Rust-only `\u{…}` escape: C universal character names use
            // `\uXXXX`/`\UXXXXXXXX`, without braces.
            let code = u32::from_str_radix(&hex.replace('_', ""), 16).ok()?;
            Some(char_to_c_literal(char::from_u32(code)?))
        } else {
            // The remaining Rust escapes (`\n`, `\t`, `\r`, `\\`, `\'`,
            // `\"`, `\0`, `\xNN`) are also valid C escapes.
            Some(expr.to_string())
        }
    } else {
        Some(char_to_c_literal(inner.chars().next()?))
    }
}

/// Render a char as a C literal: plain for printable ASCII, `\xNN` for ASCII
/// control codes, `U'\UXXXXXXXX'` for everything else.
fn char_to_c_literal(ch: char) -> String {
    match ch {
        '\'' => r"'\''".to_string(),
        '\\' => r"'\\'".to_string(),
        ch if ch.is_ascii_control() => format!("'\\x{:02X}'", ch as u32),
        ch if ch.is_ascii() => format!("'{ch}'"),
        ch => format!("U'\\U{:08X}'", ch as u32),
    }
}

/// Convert a Rust numeric literal to a C-compatible form.
///
/// Strips Rust type suffixes (`u8`, `i32`, `usize`, `f64`, …) and
/// underscores used as digit separators. For floating-point types,
/// also appends `.0` when the resulting value is otherwise indistinguishable
/// from an integer literal (no decimal point and no exponent), so that
/// C compilers infer `float`/`double` instead of `int` for the `#define`.
fn sanitize_rust_number(value: &str, prim: &ScalarPrimitive) -> String {
    let suffix = match prim {
        ScalarPrimitive::U8 => "u8",
        ScalarPrimitive::U16 => "u16",
        ScalarPrimitive::U32 => "u32",
        ScalarPrimitive::U64 => "u64",
        ScalarPrimitive::Usize => "usize",
        ScalarPrimitive::I8 => "i8",
        ScalarPrimitive::I16 => "i16",
        ScalarPrimitive::I32 => "i32",
        ScalarPrimitive::I64 => "i64",
        ScalarPrimitive::Isize => "isize",
        ScalarPrimitive::F32 => "f32",
        ScalarPrimitive::F64 => "f64",
        ScalarPrimitive::Bool
        | ScalarPrimitive::Char
        | ScalarPrimitive::Str
        | ScalarPrimitive::U128
        | ScalarPrimitive::I128 => {
            unreachable!(
                "Bool, Char, Str, U128 and I128 are handled by earlier match arms in resolve_constant"
            )
        }
    };

    let cleaned = value.strip_suffix(suffix).unwrap_or(value).replace('_', "");

    if matches!(prim, ScalarPrimitive::F32 | ScalarPrimitive::F64)
        && looks_like_integer_literal(&cleaned)
    {
        format!("{cleaned}.0")
    } else {
        cleaned
    }
}

/// Whether `s` would be parsed as an integer literal by a C compiler.
///
/// Returns `false` for empty input, anything containing a decimal point or
/// exponent marker, and non-numeric tokens such as `inf` or `NaN`.
fn looks_like_integer_literal(s: &str) -> bool {
    let digits = s.strip_prefix(['-', '+']).unwrap_or(s);
    !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())
}
