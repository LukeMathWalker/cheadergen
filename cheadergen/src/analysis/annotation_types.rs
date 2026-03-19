use rustdoc_ir::PathType;
use rustdoc_processor::queries::Crate;
use rustdoc_types::ItemEnum;

use crate::indexing::CheadergenAnnotations;

/// Build [`PathType`]s for items annotated with `#[cheadergen::export]`.
///
/// Looks up each annotated ID in the crate's import index to determine its
/// path. Items that aren't structs, enums, unions, or type aliases are
/// skipped with a warning.
pub fn annotated_path_types(
    annotations: Option<&CheadergenAnnotations>,
    krate: &Crate,
) -> Vec<PathType> {
    let Some(annotations) = annotations else {
        return Vec::new();
    };

    let mut result = Vec::new();
    for id in &annotations.exported_ids {
        let Some(item) = krate.core.krate.index.get(id) else {
            eprintln!(
                "warning: annotated item {id:?} not found in crate index, skipping"
            );
            continue;
        };

        match &item.inner {
            ItemEnum::Struct(_)
            | ItemEnum::Enum(_)
            | ItemEnum::Union(_)
            | ItemEnum::TypeAlias(_) => {}
            _ => {
                let name = item.name.as_deref().unwrap_or("<unnamed>");
                eprintln!(
                    "warning: #[cheadergen::export] on `{name}` is not a struct, enum, union, or type alias — skipping"
                );
                continue;
            }
        }

        let Some(entry) = krate.import_index.items.get(id) else {
            let name = item.name.as_deref().unwrap_or("<unnamed>");
            eprintln!(
                "warning: annotated item `{name}` not found in import index, skipping"
            );
            continue;
        };

        let path = entry.canonical_path();
        result.push(PathType {
            package_id: krate.core.package_id.clone(),
            rustdoc_id: Some(*id),
            base_type: path.to_vec(),
            generic_arguments: Vec::new(),
        });
    }

    result
}
