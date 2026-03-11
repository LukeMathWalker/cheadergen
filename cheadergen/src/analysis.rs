use std::collections::{BTreeMap, BTreeSet};

use rustdoc_ir::{FreeFunction, GenericArgument, Type};
use rustdoc_processor::CrateCollection;
use rustdoc_processor::indexing::NoAnnotations;
use rustdoc_processor::queries::Crate;
use rustdoc_resolver::resolve_free_function;
use rustdoc_types::{Abi, Attribute, ItemEnum};

/// A user-defined type that needs a C declaration in the header.
pub struct CTypeDefinition {
    /// The C name for this type (last path segment from PathType::base_type).
    pub name: String,
}

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

/// Walk all types in all function signatures and collect unique path types
/// that need forward declarations in the generated header.
pub fn collect_type_definitions(functions: &[FreeFunction]) -> Vec<CTypeDefinition> {
    let mut seen = BTreeMap::new();
    for func in functions {
        for input in &func.header.inputs {
            collect_paths_from_type(&input.type_, &mut seen);
        }
        if let Some(output) = &func.header.output {
            collect_paths_from_type(output, &mut seen);
        }
    }
    seen.into_values().collect()
}

/// Compute the cbindgen-style monomorphized C name for a type.
///
/// - Scalars use their Rust name (e.g. `"i32"`, `"bool"`).
/// - Path types use the last segment plus mangled generic arguments.
///   Generic arguments at the same level are separated by `__` (double underscore),
///   and a single `_` separates the base name from the first argument.
///   Lifetimes are ignored.
/// - References and raw pointers recurse into the inner type.
pub fn c_type_name(ty: &Type) -> String {
    match ty {
        Type::ScalarPrimitive(p) => p.as_str().to_owned(),
        Type::Path(p) => {
            let base = p.base_type.last().expect("empty path");
            let type_args: Vec<String> = p
                .generic_arguments
                .iter()
                .filter_map(|arg| match arg {
                    GenericArgument::TypeParameter(t) => Some(c_type_name(t)),
                    GenericArgument::Lifetime(_) => None,
                })
                .collect();
            if type_args.is_empty() {
                base.clone()
            } else {
                format!("{}_{}", base, type_args.join("__"))
            }
        }
        Type::Reference(r) => c_type_name(&r.inner),
        Type::RawPointer(r) => c_type_name(&r.inner),
        Type::Tuple(t) => {
            // Tuples shouldn't typically appear in monomorphized names,
            // but handle gracefully.
            let elems: Vec<String> = t.elements.iter().map(c_type_name).collect();
            elems.join("__")
        }
        Type::Slice(s) => c_type_name(&s.element_type),
        Type::Array(a) => c_type_name(&a.element_type),
        Type::Generic(g) => g.name.clone(),
    }
}

fn collect_paths_from_type(ty: &Type, seen: &mut BTreeMap<String, CTypeDefinition>) {
    match ty {
        Type::Path(_) => {
            let name = c_type_name(ty);
            seen.entry(name.clone())
                .or_insert(CTypeDefinition { name });
        }
        Type::Reference(r) => collect_paths_from_type(&r.inner, seen),
        Type::RawPointer(r) => collect_paths_from_type(&r.inner, seen),
        Type::Tuple(t) => {
            for element in &t.elements {
                collect_paths_from_type(element, seen);
            }
        }
        Type::Slice(s) => collect_paths_from_type(&s.element_type, seen),
        Type::Array(a) => collect_paths_from_type(&a.element_type, seen),
        Type::ScalarPrimitive(_) | Type::Generic(_) => {}
    }
}
