mod annotation_types;
mod c_canonical_type;
pub(crate) mod extern_items;
pub(crate) mod partitioning;
mod type_collection;
mod type_resolution;
mod type_transform;

pub use annotation_types::exported_via_annotations;
pub use c_canonical_type::CCanonicalType;
pub use extern_items::{ExternItemCoordinates, collect_symbols, find_assoc_constants};
use rustdoc_processor::GlobalItemId;
use rustdoc_processor::queries::Crate;
pub use type_collection::{
    CEnumRepr, CEnumVariant, CFieldlessEnumDef, CIdentifier, CStructDef, CStructField,
    CTaggedUnionDef, CTypeDefinition, CTypeKind, CTypedefDef, CUnionDef, c_type_name,
    collect_type_definitions, collect_type_definitions_multi, ffi_primitive_to_c,
};

use crate::Collection;
use crate::config::SortKey;

/// Trait for items that carry a [`GlobalItemId`] for sorting purposes.
pub trait HasGlobalId {
    fn global_id(&self) -> Option<&GlobalItemId>;
    fn fallback_name(&self) -> String;
}

impl HasGlobalId for CTypeDefinition {
    fn global_id(&self) -> Option<&GlobalItemId> {
        self.rustdoc_id.as_ref()
    }

    fn fallback_name(&self) -> String {
        self.name.clone()
    }
}

/// Sort extern item IDs (functions, statics, constants) using the local crate index.
///
/// These IDs are guaranteed to belong to the root crate (from `krate.import_index`),
/// so a local lookup is correct.
pub fn sort_local_ids_by_key(ids: &mut [rustdoc_types::Id], sort_by: SortKey, krate: &Crate) {
    match sort_by {
        SortKey::SourceOrder => ids.sort_by_cached_key(|id| {
            let (line, col) = span_sort_key_local(id, krate);
            (line, col, name_sort_key_local(id, krate))
        }),
        SortKey::Name => ids.sort_by_cached_key(|id| name_sort_key_local(id, krate)),
    }
}

/// Sort items that carry a [`GlobalItemId`] using the [`Collection`] for lookups.
pub fn sort_by_key<T: HasGlobalId>(items: &mut [T], sort_by: SortKey, collection: &Collection) {
    match sort_by {
        SortKey::SourceOrder => items.sort_by_cached_key(|item| {
            let (line, col) = match item.global_id() {
                Some(gid) => span_sort_key_global(gid, collection),
                None => (usize::MAX, usize::MAX),
            };
            (line, col, item.fallback_name())
        }),
        SortKey::Name => items.sort_by_cached_key(|item| match item.global_id() {
            Some(gid) => name_sort_key_global(gid, collection),
            None => item.fallback_name(),
        }),
    }
}

/// Sort key: (line, column) from the item's span, using the local crate index.
fn span_sort_key_local(id: &rustdoc_types::Id, krate: &Crate) -> (usize, usize) {
    let Some(item) = krate.core.krate.index.get(id) else {
        return (usize::MAX, usize::MAX);
    };
    match item.span.as_ref() {
        Some(span) => (span.begin.0, span.begin.1),
        None => (usize::MAX, usize::MAX),
    }
}

/// Sort key: item name, using the local crate index.
fn name_sort_key_local(id: &rustdoc_types::Id, krate: &Crate) -> String {
    krate
        .core
        .krate
        .index
        .get(id)
        .and_then(|item| item.name.clone())
        .unwrap_or_default()
}

/// Sort key: (line, column) from the item's span, using the collection for cross-crate lookup.
pub(crate) fn span_sort_key_global(gid: &GlobalItemId, collection: &Collection) -> (usize, usize) {
    let item = collection.get_item_by_global_type_id(gid);
    match item.span.as_ref() {
        Some(span) => (span.begin.0, span.begin.1),
        None => (usize::MAX, usize::MAX),
    }
}

/// Sort key: item name, using the collection for cross-crate lookup.
fn name_sort_key_global(gid: &GlobalItemId, collection: &Collection) -> String {
    let item = collection.get_item_by_global_type_id(gid);
    item.name.clone().unwrap_or_default()
}
