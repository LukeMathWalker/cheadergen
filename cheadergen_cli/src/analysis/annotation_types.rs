use std::collections::HashSet;

use guppy::PackageId;
use rustdoc_processor::compute::CannotGetCrateData;
use rustdoc_resolver::{TypeAliasResolution, rustdoc_item_def2type};
use rustdoc_types::ItemEnum;

use crate::Collection;
use crate::analysis::CCanonicalType;
use crate::diagnostic::DiagnosticSink;
use crate::indexing::ExportMode;

/// Types annotated with `#[cheadergen::config(export)]` or
/// `#[cheadergen::config(export(opaque))]` in a given crate,
/// split by export mode.
///
/// Types are stored in lifetime-erased canonical form so that "contains"
/// checks are robust against lifetime and unassigned-generic differences.
pub struct AnnotatedExports {
    #[expect(unused)]
    /// The package ID of the crate that these types belong to.
    pub package_id: PackageId,
    /// Types to include with their full definition.
    pub full: HashSet<CCanonicalType>,
    /// Types to include as opaque forward declarations only.
    pub opaque: HashSet<CCanonicalType>,
}

/// Build [`CCanonicalType`]s for items annotated with `#[cheadergen::export]`.
///
/// Looks up each annotated ID in the crate's import index to determine its
/// path. Items that aren't structs, enums, unions, or type aliases are
/// skipped with a warning.
///
/// Returns the types split by export mode: full definitions vs opaque
/// forward declarations.
pub fn exported_via_annotations(
    package_id: &PackageId,
    collection: &Collection,
    diagnostics: &mut DiagnosticSink,
) -> Result<AnnotatedExports, CannotGetCrateData> {
    let Some(annotations) = collection.get_annotated_items(package_id) else {
        return Ok(AnnotatedExports {
            full: HashSet::new(),
            opaque: HashSet::new(),
            package_id: package_id.to_owned(),
        });
    };
    let krate = collection.get_or_compute(package_id)?;
    let mut full = HashSet::new();
    let mut opaque = HashSet::new();
    for (id, ann) in &annotations.items {
        let Some(export_mode) = &ann.export else {
            continue;
        };

        let Some(item) = krate.core.krate.index.get(id) else {
            diagnostics
                .warning(format!(
                    "annotated item {id:?} not found in crate index for package {}",
                    package_id.repr()
                ))
                .emit();
            continue;
        };

        if !matches!(
            &item.inner,
            ItemEnum::Struct(_) | ItemEnum::Enum(_) | ItemEnum::Union(_) | ItemEnum::TypeAlias(_)
        ) {
            let name = item.name.as_deref().unwrap_or("<unnamed>");
            diagnostics
                .warning(format!(
                    "#[cheadergen::export] on `{name}` is not a struct, enum, union, or type alias"
                ))
                .with_span_if(item.span.as_ref())
                .emit();
            continue;
        }

        let ty =
            match rustdoc_item_def2type(&item, krate, collection, TypeAliasResolution::Preserve) {
                Ok(t) => t,
                Err(e) => {
                    let name = item.name.as_deref().unwrap_or("<unnamed>");
                    diagnostics
                        .warning(format!("failed to resolve exported type `{name}`"))
                        .with_span_if(item.span.as_ref())
                        .with_error_chain(&e)
                        .emit();
                    continue;
                }
            };
        let canonical = CCanonicalType::new(ty.canonicalize(collection));

        match export_mode {
            ExportMode::Full => full.insert(canonical),
            ExportMode::Opaque => opaque.insert(canonical),
        };
    }

    Ok(AnnotatedExports {
        package_id: package_id.to_owned(),
        full,
        opaque,
    })
}
