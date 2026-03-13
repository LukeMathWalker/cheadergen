use rustdoc_ir::{ScalarPrimitive, Type};
use rustdoc_processor::CrateCollection;
use rustdoc_processor::indexing::CrateIndexer;
use rustdoc_processor::queries::Crate;
use rustdoc_resolver::resolve_type;
use rustdoc_types::{Item, ItemEnum};

/// A resolved constant item ready for C codegen as `#define NAME VALUE`.
pub struct ConstantItem {
    /// The Rust item name (used as the `#define` macro name).
    pub name: String,
    /// The evaluated value from rustdoc (emitted verbatim after `#define`).
    pub value: String,
    /// The rustdoc item ID, used for doc comment lookup at codegen time.
    pub rustdoc_id: rustdoc_types::Id,
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

    let name = item
        .name
        .clone()
        .unwrap_or_else(|| "<unnamed>".to_string());

    let resolved = match resolve_type(
        type_,
        &krate.core.package_id,
        collection,
        &Default::default(),
    ) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("warning: constant `{name}`: failed to resolve type: {e}");
            return None;
        }
    };

    // Only emit constants whose type is a supported scalar primitive.
    let Type::ScalarPrimitive(prim) = &resolved else {
        eprintln!(
            "warning: constant `{name}` has non-primitive type; skipping"
        );
        return None;
    };

    // Skip char and str — not supported yet.
    if matches!(prim, ScalarPrimitive::Char | ScalarPrimitive::Str) {
        eprintln!(
            "warning: constant `{name}` has unsupported primitive type `{prim}`; skipping"
        );
        return None;
    }

    let Some(ref value) = const_.value else {
        eprintln!(
            "warning: constant `{name}` has no evaluated value; skipping"
        );
        return None;
    };

    Some(ConstantItem {
        name,
        value: sanitize_rust_literal(value, prim),
        rustdoc_id: item.id,
    })
}

/// Convert a Rust numeric literal to a C-compatible form.
///
/// Strips Rust type suffixes (`u8`, `i32`, `usize`, `f64`, …) and
/// underscores used as digit separators.
fn sanitize_rust_literal(value: &str, prim: &ScalarPrimitive) -> String {
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
        ScalarPrimitive::Bool => "",
        _ => "",
    };

    let stripped = if !suffix.is_empty() {
        value.strip_suffix(suffix).unwrap_or(value)
    } else {
        value
    };

    stripped.replace('_', "")
}
