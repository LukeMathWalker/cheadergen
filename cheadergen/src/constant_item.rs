use rustdoc_ir::{ScalarPrimitive, Type};
use rustdoc_processor::indexing::CrateIndexer;
use rustdoc_processor::queries::Crate;
use rustdoc_processor::{CrateCollection, GlobalItemId};
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
/// Returns `None` (with a warning on stderr) if:
/// - The type does not resolve to a [`ScalarPrimitive`] (non-primitive constants are unsupported).
/// - The constant has no evaluated `value` from rustdoc.
pub fn resolve_constant<I: CrateIndexer>(
    item: &Item,
    krate: &Crate,
    collection: &CrateCollection<I>,
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
            eprintln!("warning: constant `{name}`: failed to resolve type: {e}");
            return None;
        }
    };

    let value = match &resolved {
        // Bool: rustdoc emits "true"/"false", valid C as-is.
        Type::ScalarPrimitive(ScalarPrimitive::Bool) => {
            let Some(ref value) = const_.value else {
                eprintln!("warning: constant `{name}` has no evaluated value; skipping");
                return None;
            };
            value.clone()
        }
        // Char: rustdoc doesn't evaluate char literals, so use `expr`.
        // Only emit simple literals (not computed expressions).
        Type::ScalarPrimitive(ScalarPrimitive::Char) => {
            if !const_.is_literal {
                eprintln!("warning: constant `{name}` is a computed char expression; skipping");
                return None;
            }
            sanitize_char_literal(&const_.expr)
        }
        // Numeric types: strip Rust suffixes and underscores.
        Type::ScalarPrimitive(prim)
            if !matches!(
                prim,
                ScalarPrimitive::Str | ScalarPrimitive::U128 | ScalarPrimitive::I128
            ) =>
        {
            let Some(ref value) = const_.value else {
                eprintln!("warning: constant `{name}` has no evaluated value; skipping");
                return None;
            };
            sanitize_rust_number(value, prim)
        }
        // &str: rustdoc doesn't evaluate string literals, so use `expr`.
        // Only emit simple literals (not computed expressions).
        Type::Reference(r) if matches!(&*r.inner, Type::ScalarPrimitive(ScalarPrimitive::Str)) => {
            if !const_.is_literal {
                eprintln!("warning: constant `{name}` is a computed string expression; skipping");
                return None;
            }
            const_.expr.clone()
        }
        _ => {
            eprintln!("warning: constant `{name}` has unsupported type; skipping");
            return None;
        }
    };

    Some(ConstantItem {
        name,
        value,
        rustdoc_id: GlobalItemId::new(item.id, krate.core.package_id.clone()),
    })
}

/// Convert a Rust char literal (from rustdoc's `expr`) to a C-compatible form.
///
/// ASCII chars and standard escape sequences pass through unchanged.
/// Non-ASCII Unicode chars are converted to C11 universal character names.
fn sanitize_char_literal(expr: &str) -> String {
    // expr is like "'X'" — strip surrounding quotes
    let inner = &expr[1..expr.len() - 1];
    if inner.starts_with('\\') {
        // Escape sequence — ASCII-compatible, pass through
        expr.to_string()
    } else {
        let ch = inner.chars().next().expect("empty char literal");
        if ch.is_ascii() {
            expr.to_string()
        } else {
            format!("U'\\U{:08X}'", ch as u32)
        }
    }
}

/// Convert a Rust numeric literal to a C-compatible form.
///
/// Strips Rust type suffixes (`u8`, `i32`, `usize`, `f64`, …) and
/// underscores used as digit separators.
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

    value
        .strip_suffix(suffix)
        .unwrap_or(value)
        .replace('_', "")
}
