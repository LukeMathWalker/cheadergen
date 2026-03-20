use std::collections::BTreeSet;

use rustdoc_ir::FreeFunction;
use rustdoc_processor::queries::Crate;

use crate::Collection;
use rustdoc_resolver::{TypeAliasResolution, resolve_free_function};
use rustdoc_types::{Abi, Attribute, ItemEnum};

use super::type_transform;

use crate::analysis::CTypeDefinition;
use crate::constant_item::{ConstantItem, resolve_assoc_constant, resolve_constant};
use crate::static_item::{StaticItem, resolve_static};

/// Extern "C" function IDs, exported static IDs, and constant IDs found in a crate.
pub struct ExternItems {
    pub fn_ids: Vec<rustdoc_types::Id>,
    pub static_ids: Vec<rustdoc_types::Id>,
    pub constant_ids: Vec<rustdoc_types::Id>,
}

/// Walk the crate's import index and collect extern "C" functions, exported statics,
/// and public constants.
pub fn find_extern_items(krate: &Crate) -> ExternItems {
    let mut fn_ids = Vec::new();
    let mut static_ids = Vec::new();
    let mut constant_ids = Vec::new();

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
            ItemEnum::Constant { .. } => {
                constant_ids.push(*id);
            }
            _ => {}
        }
    }

    ExternItems {
        fn_ids,
        static_ids,
        constant_ids,
    }
}

/// Resolve each extern "C" function ID into the IR, validating types along the way.
pub fn resolve_functions(
    fn_ids: &[rustdoc_types::Id],
    krate: &Crate,
    collection: &Collection,
) -> anyhow::Result<Vec<FreeFunction>> {
    let mut resolved_fns = Vec::new();
    for id in fn_ids {
        let item = krate
            .core
            .krate
            .index
            .get(id)
            .ok_or_else(|| anyhow::anyhow!("Missing item for id {:?}", id))?;
        let mut free_fn = resolve_free_function(&item, krate, collection, TypeAliasResolution::Preserve)
            .map_err(|e| anyhow::anyhow!("Failed to resolve function: {e}"))?;

        for input in &mut free_fn.header.inputs {
            type_transform::simplify_type(&mut input.type_, collection);
        }
        if let Some(output) = &mut free_fn.header.output {
            type_transform::simplify_type(output, collection);
        }

        resolved_fns.push(free_fn);
    }
    Ok(resolved_fns)
}

/// Resolve each exported static ID into a [`StaticItem`].
pub fn resolve_statics(
    static_ids: &[rustdoc_types::Id],
    krate: &Crate,
    collection: &Collection,
) -> anyhow::Result<Vec<StaticItem>> {
    let mut resolved = Vec::new();
    for id in static_ids {
        let item = krate
            .core
            .krate
            .index
            .get(id)
            .ok_or_else(|| anyhow::anyhow!("Missing item for id {:?}", id))?;
        let mut static_item = resolve_static(&item, krate, collection)
            .map_err(|e| anyhow::anyhow!("Failed to resolve static: {e}"))?;
        type_transform::simplify_type(&mut static_item.type_, collection);
        resolved.push(static_item);
    }
    Ok(resolved)
}

/// Resolve each constant ID into a [`ConstantItem`], skipping unsupported types.
pub fn resolve_constants(
    constant_ids: &[rustdoc_types::Id],
    krate: &Crate,
    collection: &Collection,
) -> Vec<ConstantItem> {
    let mut resolved = Vec::new();
    for id in constant_ids {
        let Some(item) = krate.core.krate.index.get(id) else {
            continue;
        };
        if let Some(constant) = resolve_constant(&item, krate, collection) {
            resolved.push(constant);
        }
    }
    resolved
}

/// Extract symbol names from function and static IDs.
pub fn collect_symbols(items: &ExternItems, krate: &Crate) -> BTreeSet<String> {
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

/// Find associated constants on each type definition.
///
/// For each `CTypeDefinition` with a `rustdoc_id`, this looks up the struct/enum/union
/// in the crate index, walks its inherent `impl` blocks, and resolves public
/// `AssocConst` items. Returns a vec of `(type_name, Vec<ConstantItem>)` pairs,
/// preserving the order of `type_defs`.
pub fn find_assoc_constants(
    type_defs: &[CTypeDefinition],
    krate: &Crate,
    collection: &Collection,
) -> Vec<(String, Vec<ConstantItem>)> {
    let mut result = Vec::new();

    for def in type_defs {
        let Some(ref global_id) = def.rustdoc_id else {
            continue;
        };
        let Some(item) = krate.core.krate.index.get(&global_id.rustdoc_item_id) else {
            continue;
        };

        // Extract impl IDs from the struct/enum/union.
        let impl_ids: &[rustdoc_types::Id] = match &item.inner {
            ItemEnum::Struct(s) => &s.impls,
            ItemEnum::Enum(e) => &e.impls,
            ItemEnum::Union(u) => &u.impls,
            _ => continue,
        };

        let mut constants = Vec::new();
        for impl_id in impl_ids {
            let Some(impl_item) = krate.core.krate.index.get(impl_id) else {
                continue;
            };
            let ItemEnum::Impl(ref impl_def) = impl_item.inner else {
                continue;
            };
            // Skip trait impls — we only want inherent impls.
            if impl_def.trait_.is_some() {
                continue;
            }

            for assoc_id in &impl_def.items {
                let Some(assoc_item) = krate.core.krate.index.get(assoc_id) else {
                    continue;
                };
                // Only public associated constants.
                if !matches!(assoc_item.visibility, rustdoc_types::Visibility::Public) {
                    continue;
                }
                if !matches!(assoc_item.inner, ItemEnum::AssocConst { .. }) {
                    continue;
                }
                if let Some(c) = resolve_assoc_constant(&assoc_item, &def.name, krate, collection) {
                    constants.push(c);
                }
            }
        }

        if !constants.is_empty() {
            result.push((def.name.clone(), constants));
        }
    }

    result
}

/// Returns `true` if the item has `#[no_mangle]` or `#[export_name = "..."]`.
fn has_export_attr(attrs: &[Attribute]) -> bool {
    attrs
        .iter()
        .any(|a| matches!(a, Attribute::NoMangle | Attribute::ExportName(_)))
}
