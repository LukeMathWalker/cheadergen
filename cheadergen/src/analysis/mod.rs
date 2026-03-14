mod extern_items;
mod type_collection;
mod type_resolution;

pub use extern_items::{
    collect_symbols, find_extern_items, resolve_constants, resolve_functions, resolve_statics,
};
pub use type_collection::{
    CEnumRepr, CEnumVariant, CFieldlessEnumDef, CIdentifier, CStructDef, CTaggedUnionDef,
    CTypeDefinition, CTypeKind, c_type_name, collect_type_definitions,
};

use rustdoc_processor::queries::Crate;

use crate::config::SortKey;

/// Trait for items that can be sorted by their rustdoc ID.
pub trait HasRustdocId {
    fn rustdoc_id(&self) -> Option<&rustdoc_types::Id>;
    fn fallback_name(&self) -> String;
}

impl HasRustdocId for rustdoc_types::Id {
    fn rustdoc_id(&self) -> Option<&rustdoc_types::Id> {
        Some(self)
    }

    fn fallback_name(&self) -> String {
        String::new()
    }
}

impl HasRustdocId for CTypeDefinition {
    fn rustdoc_id(&self) -> Option<&rustdoc_types::Id> {
        self.rustdoc_id.as_ref()
    }

    fn fallback_name(&self) -> String {
        self.name.clone()
    }
}

/// Sort a slice of items according to the given [`SortKey`], using each
/// item's rustdoc ID to look up source position or name.
pub fn sort_by_key<T: HasRustdocId>(items: &mut [T], sort_by: SortKey, krate: &Crate) {
    match sort_by {
        SortKey::SourceOrder => items.sort_by_cached_key(|item| {
            let (line, col) = match item.rustdoc_id() {
                Some(id) => span_sort_key(id, krate),
                None => (usize::MAX, usize::MAX),
            };
            // Tiebreak on the item's own name so that multiple
            // monomorphisations of the same generic (same span) sort
            // deterministically.
            (line, col, item.fallback_name())
        }),
        SortKey::Name => items.sort_by_cached_key(|item| match item.rustdoc_id() {
            Some(id) => name_sort_key(id, krate),
            None => item.fallback_name(),
        }),
    }
}

/// Sort key: (line, column) from the item's span.
fn span_sort_key(id: &rustdoc_types::Id, krate: &Crate) -> (usize, usize) {
    let Some(item) = krate.core.krate.index.get(id) else {
        return (usize::MAX, usize::MAX);
    };
    match item.span.as_ref() {
        Some(span) => (span.begin.0, span.begin.1),
        None => (usize::MAX, usize::MAX),
    }
}

/// Sort key: item name, alphabetically.
fn name_sort_key(id: &rustdoc_types::Id, krate: &Crate) -> String {
    krate
        .core
        .krate
        .index
        .get(id)
        .and_then(|item| item.name.clone())
        .unwrap_or_default()
}
