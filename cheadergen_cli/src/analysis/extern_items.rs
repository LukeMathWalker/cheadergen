use std::collections::BTreeSet;

use guppy::PackageId;
use rustdoc_ir::FreeFunction;
use rustdoc_processor::compute::CannotGetCrateData;
use rustdoc_processor::queries::Crate;

use crate::Collection;
use crate::config::CommonConfig;
use crate::diagnostic::DiagnosticSink;
use rustdoc_resolver::{CallableResolutionError, TypeAliasResolution, resolve_free_function};
use rustdoc_types::{Abi, Attribute, ItemEnum};

use super::type_transform;

use crate::analysis::{CTypeDefinition, sort_local_ids_by_key};
use crate::constant_item::{ConstantItem, resolve_assoc_constant, resolve_constant};
use crate::static_item::{StaticItem, resolve_static};

/// Extern "C" function IDs, exported static IDs, and constant IDs found in a crate.
pub struct ExternItemCoordinates {
    /// The package ID of the crate that all these items belong to.
    pub package_id: PackageId,
    /// The IDs of extern "C" functions found in the crate.
    pub fn_ids: Vec<rustdoc_types::Id>,
    /// The IDs of exported statics found in the crate.
    pub static_ids: Vec<rustdoc_types::Id>,
    /// The IDs of public constants found in the crate.
    pub constant_ids: Vec<rustdoc_types::Id>,
}

impl ExternItemCoordinates {
    /// Walk the crate's import index and collect the IDs of extern "C" functions, exported statics,
    /// and public constants.
    pub fn collect(
        collection: &Collection,
        package_id: &PackageId,
    ) -> Result<Self, CannotGetCrateData> {
        let krate = collection.get_or_compute(package_id)?;
        let annotations = collection.get_annotated_items(package_id);

        let mut fn_ids = Vec::new();
        let mut static_ids = Vec::new();
        let mut constant_ids = Vec::new();
        for id in krate.import_index.items.keys() {
            let Some(item) = krate.core.krate.index.get(id) else {
                continue;
            };

            // Filter out items annotated with `#[cheadergen::config(skip)]`.
            if let Some(ann) = annotations.as_ref().and_then(|a| a.get(id))
                && ann.skip
            {
                continue;
            }

            match &item.inner {
                ItemEnum::Function(func)
                    if matches!(func.header.abi, Abi::C { .. }) && func.has_body =>
                {
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

        Ok(ExternItemCoordinates {
            package_id: package_id.to_owned(),
            fn_ids,
            static_ids,
            constant_ids,
        })
    }

    /// Resolve each extern item id into the IR, validating types along the way.
    pub fn resolve(
        self,
        collection: &Collection,
        config: &CommonConfig,
        diagnostics: &mut DiagnosticSink,
    ) -> ExternItems {
        let krate = collection
            .get_or_compute(&self.package_id)
            .expect("We computed this crate's doc earlier on, when collecting ids of extern items");

        let Self {
            package_id,
            mut fn_ids,
            mut static_ids,
            mut constant_ids,
        } = self;

        // Sort IDs before resolution — resolvers preserve input order,
        // so the output inherits the sort.
        sort_local_ids_by_key(&mut fn_ids, config.fn_sort_by, krate);
        sort_local_ids_by_key(&mut static_ids, config.static_sort_by, krate);
        sort_local_ids_by_key(&mut constant_ids, config.constant_sort_by, krate);

        let fns = resolve_functions(&fn_ids, krate, collection, diagnostics);
        let statics = resolve_statics(&static_ids, krate, collection, diagnostics);
        let constants = resolve_constants(&constant_ids, krate, collection, diagnostics);

        ExternItems {
            package_id,
            fns,
            statics,
            constants,
        }
    }
}

/// Extern "C" function, exported static, and constant found in a crate.
pub struct ExternItems {
    /// The package ID of the crate that all these items belong to.
    pub package_id: PackageId,
    /// The extern "C" functions found in the crate.
    pub fns: Vec<FreeFunction>,
    /// The exported statics found in the crate.
    pub statics: Vec<StaticItem>,
    /// The public constants found in the crate.
    pub constants: Vec<ConstantItem>,
}

/// Resolve each extern "C" function ID into the IR, validating types along the way.
///
/// On error, pushes a diagnostic and skips the function rather than aborting.
fn resolve_functions(
    fn_ids: &[rustdoc_types::Id],
    krate: &Crate,
    collection: &Collection,
    diagnostics: &mut DiagnosticSink,
) -> Vec<FreeFunction> {
    let mut resolved_fns = Vec::new();
    for id in fn_ids {
        let Some(item) = krate.core.krate.index.get(id) else {
            diagnostics
                .error(format!("missing item for function id {id:?}"))
                .emit();
            continue;
        };
        let name = item.name.as_deref().unwrap_or("<unnamed>");
        let ItemEnum::Function(func_inner) = &item.inner else {
            continue;
        };
        let mut free_fn = match resolve_free_function(
            &item,
            krate,
            collection,
            TypeAliasResolution::Preserve,
        ) {
            Ok(f) => f,
            Err(e) => {
                let (msg, source): (String, &dyn std::error::Error) = match &e {
                    CallableResolutionError::InputParameterResolutionError(inner) => {
                        let param_name = func_inner
                            .sig
                            .inputs
                            .get(inner.parameter_index)
                            .map(|(name, _)| name.as_str())
                            .unwrap_or("?");
                        (
                            format!(
                                "failed to resolve type of parameter `{param_name}` in function `{name}`"
                            ),
                            (*inner.source).as_ref() as _,
                        )
                    }
                    CallableResolutionError::OutputTypeResolutionError(inner) => (
                        format!("failed to resolve return type of function `{name}`"),
                        (*inner.source).as_ref() as _,
                    ),
                    CallableResolutionError::SelfResolutionError(inner) => (
                        format!("failed to resolve `Self` type for `{name}`"),
                        (*inner.source).as_ref() as _,
                    ),
                };
                diagnostics
                    .error(msg)
                    .with_span_if(item.span.as_ref())
                    .with_error_chain(source)
                    .emit();
                continue;
            }
        };

        for input in &mut free_fn.header.inputs {
            type_transform::simplify_type(&mut input.type_, collection);
        }
        if let Some(output) = &mut free_fn.header.output {
            type_transform::simplify_type(output, collection);
        }

        resolved_fns.push(free_fn);
    }
    resolved_fns
}

/// Resolve each exported static ID into a [`StaticItem`].
///
/// On error, pushes a diagnostic and skips the static rather than aborting.
fn resolve_statics(
    static_ids: &[rustdoc_types::Id],
    krate: &Crate,
    collection: &Collection,
    diagnostics: &mut DiagnosticSink,
) -> Vec<StaticItem> {
    let mut resolved = Vec::new();
    for id in static_ids {
        let Some(item) = krate.core.krate.index.get(id) else {
            diagnostics
                .error(format!("missing item for static id {id:?}"))
                .emit();
            continue;
        };
        let name = item.name.as_deref().unwrap_or("<unnamed>");
        let mut static_item = match resolve_static(&item, krate, collection) {
            Ok(s) => s,
            Err(e) => {
                diagnostics
                    .error(format!("failed to resolve static `{name}`"))
                    .with_span_if(item.span.as_ref())
                    .with_error_chain(&*e.source)
                    .emit();
                continue;
            }
        };
        type_transform::simplify_type(&mut static_item.type_, collection);
        resolved.push(static_item);
    }
    resolved
}

/// Resolve each constant ID into a [`ConstantItem`], skipping unsupported types.
fn resolve_constants(
    constant_ids: &[rustdoc_types::Id],
    krate: &Crate,
    collection: &Collection,
    diagnostics: &mut DiagnosticSink,
) -> Vec<ConstantItem> {
    let mut resolved = Vec::new();
    for id in constant_ids {
        let Some(item) = krate.core.krate.index.get(id) else {
            continue;
        };
        if let Some(constant) = resolve_constant(&item, krate, collection, diagnostics) {
            resolved.push(constant);
        }
    }
    resolved
}

/// Extract symbol names from function and static IDs.
pub fn collect_symbols(items: &ExternItemCoordinates, krate: &Crate) -> BTreeSet<String> {
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
    diagnostics: &mut DiagnosticSink,
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
                if let Some(c) =
                    resolve_assoc_constant(&assoc_item, &def.name, krate, collection, diagnostics)
                {
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
