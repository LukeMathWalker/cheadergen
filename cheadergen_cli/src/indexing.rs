use std::collections::BTreeSet;

use guppy::PackageId;
use rustdoc_processor::crate_data::{
    CrateData, CrateItemIndex, CrateItemPaths, EagerCrateItemIndex, EagerCrateItemPaths,
};
use rustdoc_processor::indexing::{CrateIndexer, IndexResult, IndexingVisitor};
use rustdoc_processor::queries::Crate;
use rustdoc_types::Attribute;

/// The set of item IDs annotated with `#[cheadergen::export]`.
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CheadergenAnnotations {
    pub exported_ids: BTreeSet<rustdoc_types::Id>,
}

impl bincode::Decode<()> for CheadergenAnnotations {
    fn decode<D: bincode::de::Decoder<Context = ()>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let ids: BTreeSet<rustdoc_types::Id> = bincode::Decode::decode(decoder)?;
        Ok(Self { exported_ids: ids })
    }
}

/// Indexer that discovers `#[cheadergen::export]` annotations during crate
/// traversal.
pub struct CheadergenIndexer;

impl CrateIndexer for CheadergenIndexer {
    type Annotations = CheadergenAnnotations;

    fn index_raw(
        &self,
        krate: rustdoc_types::Crate,
        package_id: PackageId,
    ) -> IndexResult<CheadergenAnnotations> {
        let crate_data = CrateData {
            root_item_id: krate.root,
            index: CrateItemIndex::Eager(EagerCrateItemIndex { index: krate.index }),
            external_crates: krate.external_crates,
            format_version: krate.format_version,
            paths: CrateItemPaths::Eager(EagerCrateItemPaths { paths: krate.paths }),
        };
        self.index(crate_data, package_id)
    }

    fn index(
        &self,
        crate_data: CrateData,
        package_id: PackageId,
    ) -> IndexResult<CheadergenAnnotations> {
        let mut visitor = CheadergenVisitor::default();
        let krate = Crate::index(crate_data, package_id, &mut visitor);
        IndexResult {
            krate,
            annotations: CheadergenAnnotations {
                exported_ids: visitor.exported_ids,
            },
            can_cache_indexes: true,
        }
    }
}

/// Visitor that checks each item for `#[diagnostic::cheadergen::export]`.
#[derive(Default)]
struct CheadergenVisitor {
    exported_ids: BTreeSet<rustdoc_types::Id>,
}

impl IndexingVisitor for CheadergenVisitor {
    fn on_item_discovered(&mut self, item: &rustdoc_types::Item, item_id: rustdoc_types::Id) {
        for attr in &item.attrs {
            if let Attribute::Other(s) = attr
                && s.contains("diagnostic::cheadergen::export")
            {
                self.exported_ids.insert(item_id);
                return;
            }
        }
    }

    fn on_type_indexed(&mut self, _item_id: rustdoc_types::Id) {}
}
