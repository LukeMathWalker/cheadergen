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

use std::path::PathBuf;

use crate::Collection;
use crate::config::SortKey;

/// Sort key for `source_order`: `(span_missing, filename, line, column)`.
///
/// The filename comes first so that items from different source files never
/// interleave by line number. Items without a span sort last (`span_missing`
/// is `true`); the caller's trailing name component keeps their relative
/// order deterministic.
pub(crate) type SpanSortKey = (bool, PathBuf, usize, usize);

fn span_key(span: &rustdoc_types::Span) -> SpanSortKey {
    (false, span.filename.clone(), span.begin.0, span.begin.1)
}

pub(crate) fn missing_span_key() -> SpanSortKey {
    (true, PathBuf::new(), 0, 0)
}

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
            (
                span_sort_key_local(id, krate),
                name_sort_key_local(id, krate),
            )
        }),
        SortKey::Name => ids.sort_by_cached_key(|id| name_sort_key_local(id, krate)),
    }
}

/// Sort items that carry a [`GlobalItemId`] using the [`Collection`] for lookups.
pub fn sort_by_key<T: HasGlobalId>(items: &mut [T], sort_by: SortKey, collection: &Collection) {
    match sort_by {
        SortKey::SourceOrder => items.sort_by_cached_key(|item| {
            let span_key = match item.global_id() {
                Some(gid) => span_sort_key_global(gid, collection),
                None => missing_span_key(),
            };
            (span_key, item.fallback_name())
        }),
        SortKey::Name => items.sort_by_cached_key(|item| match item.global_id() {
            Some(gid) => name_sort_key_global(gid, collection),
            None => item.fallback_name(),
        }),
    }
}

/// Sort key: (filename, line, column) from the item's span, using the local crate index.
fn span_sort_key_local(id: &rustdoc_types::Id, krate: &Crate) -> SpanSortKey {
    let Some(item) = krate.core.krate.index.get(id) else {
        return missing_span_key();
    };
    item.span.as_ref().map_or_else(missing_span_key, span_key)
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

/// Sort key: (filename, line, column) from the item's span, using the collection for cross-crate lookup.
pub(crate) fn span_sort_key_global(gid: &GlobalItemId, collection: &Collection) -> SpanSortKey {
    let item = collection.get_item_by_global_type_id(gid);
    item.span.as_ref().map_or_else(missing_span_key, span_key)
}

/// Sort key: item name, using the collection for cross-crate lookup.
fn name_sort_key_global(gid: &GlobalItemId, collection: &Collection) -> String {
    let item = collection.get_item_by_global_type_id(gid);
    item.name.clone().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span(filename: &str, line: usize, column: usize) -> rustdoc_types::Span {
        rustdoc_types::Span {
            filename: PathBuf::from(filename),
            begin: (line, column),
            end: (line, column),
        }
    }

    /// The leading `span_missing` flag must dominate every other component, so
    /// that a span-less item sorts after *any* spanned item — even one at the
    /// very end of the last file. Guards against reordering the tuple or
    /// flipping the flag's polarity.
    #[test]
    fn missing_spans_sort_after_spanned_items() {
        assert!(missing_span_key() > span_key(&span("a.rs", 1, 1)));
        assert!(missing_span_key() > span_key(&span("zzz.rs", usize::MAX, usize::MAX)));
    }

    /// The filename precedes the line number, so items from different source
    /// files never interleave by line.
    #[test]
    fn filename_outranks_position() {
        assert!(span_key(&span("a.rs", 999, 0)) < span_key(&span("b.rs", 1, 0)));
    }

    /// Within one file, ordering falls back to line then column.
    #[test]
    fn same_file_orders_by_line_then_column() {
        assert!(span_key(&span("a.rs", 1, 0)) < span_key(&span("a.rs", 2, 0)));
        assert!(span_key(&span("a.rs", 1, 0)) < span_key(&span("a.rs", 1, 5)));
    }
}
