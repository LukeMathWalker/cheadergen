use std::collections::BTreeSet;

use rustdoc_ir::{FreeFunction, Type};
use rustdoc_processor::CrateCollection;
use rustdoc_processor::indexing::NoAnnotations;
use rustdoc_processor::queries::Crate;
use rustdoc_resolver::resolve_free_function;
use rustdoc_types::{Abi, Attribute, ItemEnum};

/// Extern "C" function IDs and exported static IDs found in a crate.
pub struct ExternItems {
    pub fn_ids: Vec<rustdoc_types::Id>,
    pub static_ids: Vec<rustdoc_types::Id>,
}

/// Walk the crate's import index and collect extern "C" functions and exported statics.
pub fn find_extern_items(krate: &Crate) -> ExternItems {
    let mut fn_ids = Vec::new();
    let mut static_ids = Vec::new();

    for id in krate.import_index.items.keys() {
        let Some(item) = krate.core.krate.index.get(id) else {
            continue;
        };
        match &item.inner {
            ItemEnum::Function(func) if matches!(func.header.abi, Abi::C { .. }) => {
                fn_ids.push(*id);
            }
            ItemEnum::Static(_) if has_export_attr(&item.attrs) => {
                static_ids.push(*id);
            }
            _ => {}
        }
    }

    ExternItems { fn_ids, static_ids }
}

/// Resolve each extern "C" function ID into the IR, validating types along the way.
pub fn resolve_functions(
    fn_ids: &[rustdoc_types::Id],
    krate: &Crate,
    collection: &CrateCollection<NoAnnotations>,
) -> anyhow::Result<Vec<FreeFunction>> {
    let mut resolved_fns = Vec::new();
    for id in fn_ids {
        let item = krate
            .core
            .krate
            .index
            .get(id)
            .ok_or_else(|| anyhow::anyhow!("Missing item for id {:?}", id))?;
        let free_fn = resolve_free_function(&item, krate, collection)
            .map_err(|e| anyhow::anyhow!("Failed to resolve function: {e}"))?;

        // Bail if any input or output type contains a PathType — we don't handle those yet.
        for input in &free_fn.header.inputs {
            reject_path_types(&input.type_, &free_fn.path.function_name, &input.name)?;
        }
        if let Some(output) = &free_fn.header.output {
            reject_path_types(output, &free_fn.path.function_name, &"return type")?;
        }

        resolved_fns.push(free_fn);
    }
    Ok(resolved_fns)
}

/// Extract symbol names from function and static IDs.
pub fn collect_symbols(
    items: &ExternItems,
    krate: &Crate,
) -> BTreeSet<String> {
    let mut symbols = BTreeSet::new();
    for id in items.fn_ids.iter().chain(&items.static_ids) {
        let Some(item) = krate.core.krate.index.get(id) else {
            continue;
        };
        if let Some(name) = exported_symbol_name(&item) {
            symbols.insert(name.to_owned());
        }
    }
    symbols
}

/// Return the linker-visible symbol name for an item.
///
/// Priority: `#[export_name = "..."]` > `item.name` (for `#[no_mangle]`).
fn exported_symbol_name(item: &rustdoc_types::Item) -> Option<&str> {
    for attr in &item.attrs {
        if let Attribute::ExportName(name) = attr {
            return Some(name);
        }
    }
    item.name.as_deref()
}

/// Returns `true` if the item has `#[no_mangle]` or `#[export_name = "..."]`.
fn has_export_attr(attrs: &[Attribute]) -> bool {
    attrs
        .iter()
        .any(|a| matches!(a, Attribute::NoMangle | Attribute::ExportName(_)))
}

/// Bail if `ty` contains a [`rustdoc_ir::PathType`] anywhere — we don't handle named/user-defined
/// types yet.
fn reject_path_types(
    ty: &Type,
    fn_name: &str,
    context: &dyn std::fmt::Display,
) -> anyhow::Result<()> {
    match ty {
        Type::Path(p) => {
            anyhow::bail!(
                "`{fn_name}`: {context} uses named type `{}`, which is not yet supported",
                p.base_type.join("::")
            );
        }
        Type::Reference(r) => reject_path_types(&r.inner, fn_name, context),
        Type::RawPointer(r) => reject_path_types(&r.inner, fn_name, context),
        Type::Tuple(t) => {
            for element in &t.elements {
                reject_path_types(element, fn_name, context)?;
            }
            Ok(())
        }
        Type::Slice(s) => reject_path_types(&s.element_type, fn_name, context),
        Type::Array(a) => reject_path_types(&a.element_type, fn_name, context),
        Type::ScalarPrimitive(_) | Type::Generic(_) => Ok(()),
    }
}
