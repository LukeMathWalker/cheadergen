use std::sync::Arc;

use rustdoc_ir::Type;
use rustdoc_processor::indexing::CrateIndexer;
use rustdoc_processor::queries::Crate;
use rustdoc_processor::{CrateCollection, GlobalItemId};
use rustdoc_resolver::resolve_type;
use rustdoc_types::{Attribute, Item, ItemEnum};

/// A resolved static item ready for C codegen.
pub struct StaticItem {
    /// The Rust item name.
    pub name: String,
    /// `#[export_name]` override, or the item name for `#[no_mangle]`.
    pub symbol_name: Option<String>,
    /// The resolved type.
    pub type_: Type,
    /// `true` for `static mut`.
    pub is_mutable: bool,
    /// The global rustdoc item ID, used for doc comment lookup at codegen time.
    pub rustdoc_id: GlobalItemId,
}

/// Convert a static item from `rustdoc_types` into a [`StaticItem`].
pub fn resolve_static<I: CrateIndexer>(
    item: &Item,
    krate: &Crate,
    collection: &CrateCollection<I>,
) -> Result<StaticItem, StaticResolutionError> {
    let ItemEnum::Static(inner) = &item.inner else {
        unreachable!("Expected a static item");
    };

    let name = item.name.clone().unwrap_or_else(|| "<unnamed>".to_string());

    let type_ = resolve_type(
        &inner.type_,
        &krate.core.package_id,
        collection,
        &Default::default(),
    )
    .map_err(|e| StaticResolutionError {
        static_name: name.clone(),
        source: Arc::new(e),
    })?;

    let symbol_name = item.attrs.iter().find_map(|attr| match attr {
        Attribute::NoMangle => item.name.clone(),
        Attribute::ExportName(n) => Some(n.clone()),
        _ => None,
    });

    Ok(StaticItem {
        name,
        symbol_name,
        type_,
        is_mutable: inner.is_mutable,
        rustdoc_id: GlobalItemId::new(item.id, krate.core.package_id.clone()),
    })
}

#[derive(Debug)]
pub struct StaticResolutionError {
    pub static_name: String,
    pub source: Arc<dyn std::error::Error + Send + Sync>,
}

impl std::fmt::Display for StaticResolutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to resolve static `{}`: {}",
            self.static_name, self.source
        )
    }
}

impl std::error::Error for StaticResolutionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&*self.source)
    }
}
