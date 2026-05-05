use std::collections::BTreeMap;

use guppy::PackageId;
use rustdoc_processor::crate_data::{
    CrateData, CrateItemIndex, CrateItemPaths, EagerCrateItemIndex, EagerCrateItemPaths,
};
use rustdoc_processor::indexing::{CrateIndexer, IndexResult, IndexingVisitor};
use rustdoc_processor::queries::Crate;
use rustdoc_types::Attribute;

/// Whether a type should be exported with its full definition or as an opaque
/// forward declaration.
#[derive(
    Copy,
    Clone,
    Debug,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
)]
pub enum ExportMode {
    Full,
    Opaque,
}

/// Casing rule used by `rename_all` and `rename_all_fields`.
#[derive(
    Copy,
    Clone,
    Debug,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
)]
#[expect(
    clippy::enum_variant_names,
    reason = "the variant names mirror the canonical serde casing literals"
)]
pub enum RenameRule {
    PascalCase,
    CamelCase,
    SnakeCase,
    ScreamingSnakeCase,
}

impl RenameRule {
    /// Parse the canonical literal embedded in the diagnostic attribute (e.g. `"camelCase"`).
    pub fn from_diagnostic_str(s: &str) -> Option<Self> {
        match s {
            "PascalCase" => Some(Self::PascalCase),
            "camelCase" => Some(Self::CamelCase),
            "snake_case" => Some(Self::SnakeCase),
            "SCREAMING_SNAKE_CASE" => Some(Self::ScreamingSnakeCase),
            _ => None,
        }
    }

    /// Apply the casing rule to a Rust identifier.
    pub fn apply(&self, ident: &str) -> String {
        use heck::{ToLowerCamelCase, ToShoutySnakeCase, ToSnakeCase, ToUpperCamelCase};
        match self {
            Self::PascalCase => ident.to_upper_camel_case(),
            Self::CamelCase => ident.to_lower_camel_case(),
            Self::SnakeCase => ident.to_snake_case(),
            Self::ScreamingSnakeCase => ident.to_shouty_snake_case(),
        }
    }
}

/// Per-item annotation directives extracted from `#[cheadergen::config(...)]`.
#[derive(
    Clone, Debug, Default, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode,
)]
pub struct ItemAnnotation {
    /// If set, force-include this type in the header.
    pub export: Option<ExportMode>,
    /// If true, exclude this item from the header.
    pub skip: bool,
    /// Override the C name emitted in the header.
    pub rename: Option<String>,
    /// Override the global `prefix_with_name` setting for this enum.
    pub prefix_with_name: Option<bool>,
    /// Assign C field names to positional fields of a tuple struct.
    pub field_names: Option<Vec<String>>,
    /// Bulk-rename fields (struct/union) or variants (enum) using a casing rule.
    pub rename_all: Option<RenameRule>,
    /// Bulk-rename fields inside struct variants of an enum.
    pub rename_all_fields: Option<RenameRule>,
}

/// Annotations extracted from `#[cheadergen::config(...)]` attributes during
/// crate indexing.
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CHeadergenAnnotations {
    /// Per-item annotations keyed by rustdoc item ID.
    pub items: BTreeMap<rustdoc_types::Id, ItemAnnotation>,
}

impl CHeadergenAnnotations {
    /// Returns the annotation for a given item ID, if any.
    pub fn get(&self, id: &rustdoc_types::Id) -> Option<&ItemAnnotation> {
        self.items.get(id)
    }
}

/// Indexer that discovers `#[cheadergen::config(...)]` annotations during crate
/// traversal.
pub struct CheadergenIndexer;

impl CrateIndexer for CheadergenIndexer {
    type Annotations = CHeadergenAnnotations;

    fn index_raw(
        &self,
        krate: rustdoc_types::Crate,
        package_id: PackageId,
    ) -> IndexResult<CHeadergenAnnotations> {
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
    ) -> IndexResult<CHeadergenAnnotations> {
        let mut visitor = CheadergenVisitor::default();
        let krate = Crate::index(crate_data, package_id, &mut visitor);
        IndexResult {
            krate,
            annotations: CHeadergenAnnotations {
                items: visitor.items,
            },
            can_cache_indexes: true,
        }
    }
}

/// The `diagnostic::cheadergen::` prefix that the proc-macro emits in attribute
/// strings.
const ATTR_PREFIX: &str = "#[diagnostic::cheadergen::";

/// Parse a single `#[diagnostic::cheadergen::...]` attribute string and apply
/// the directive to the given [`ItemAnnotation`].
///
/// Returns `true` if the string was recognized as a cheadergen attribute.
fn parse_cheadergen_attr(s: &str, ann: &mut ItemAnnotation) -> bool {
    let Some(rest) = s.strip_prefix(ATTR_PREFIX) else {
        return false;
    };
    // Strip trailing `]`
    let rest = rest.strip_suffix(']').unwrap_or(rest);

    if rest == "export" {
        ann.export = Some(ExportMode::Full);
    } else if rest == "export(opaque)" {
        ann.export = Some(ExportMode::Opaque);
    } else if rest == "skip" {
        ann.skip = true;
    } else if let Some(inner) = rest.strip_prefix("rename(") {
        // rename("Foo") — the name is quoted in the attribute string
        let inner = inner.strip_suffix(')').unwrap_or(inner);
        let name = inner.trim_matches('"');
        ann.rename = Some(name.to_owned());
    } else if rest == "prefix_with_name" || rest == "prefix_with_name(true)" {
        ann.prefix_with_name = Some(true);
    } else if rest == "prefix_with_name(false)" {
        ann.prefix_with_name = Some(false);
    } else if let Some(inner) = rest.strip_prefix("field_names(") {
        let inner = inner.strip_suffix(')').unwrap_or(inner);
        let names: Vec<String> = inner.split(',').map(|n| n.trim().to_owned()).collect();
        ann.field_names = Some(names);
    } else if let Some(inner) = rest.strip_prefix("rename_all(") {
        let inner = inner.strip_suffix(')').unwrap_or(inner).trim_matches('"');
        ann.rename_all = RenameRule::from_diagnostic_str(inner);
    } else if let Some(inner) = rest.strip_prefix("rename_all_fields(") {
        let inner = inner.strip_suffix(')').unwrap_or(inner).trim_matches('"');
        ann.rename_all_fields = RenameRule::from_diagnostic_str(inner);
    } else {
        return false;
    }
    true
}

/// Visitor that extracts `#[diagnostic::cheadergen::...]` attributes from items.
#[derive(Default)]
struct CheadergenVisitor {
    items: BTreeMap<rustdoc_types::Id, ItemAnnotation>,
}

impl IndexingVisitor for CheadergenVisitor {
    fn on_item_discovered(&mut self, item: &rustdoc_types::Item, item_id: rustdoc_types::Id) {
        let ann = item_annotation_from_attrs(&item.attrs);
        if !ann.is_empty() {
            self.items.insert(item_id, ann);
        }
    }

    fn on_type_indexed(&mut self, _item_id: rustdoc_types::Id) {}
}

/// Parse all `#[diagnostic::cheadergen::...]` attributes from a list of rustdoc
/// attributes into an [`ItemAnnotation`].
///
/// Used during indexing for top-level items, and on demand for items the
/// indexer doesn't visit (e.g. associated constants inside `impl` blocks).
pub fn item_annotation_from_attrs(attrs: &[Attribute]) -> ItemAnnotation {
    let mut ann = ItemAnnotation::default();
    for attr in attrs {
        if let Attribute::Other(s) = attr {
            parse_cheadergen_attr(s, &mut ann);
        }
    }
    ann
}

impl ItemAnnotation {
    /// Returns `true` if no cheadergen directives were applied to this annotation.
    pub fn is_empty(&self) -> bool {
        let Self {
            export,
            skip,
            rename,
            prefix_with_name,
            field_names,
            rename_all,
            rename_all_fields,
        } = self;
        export.is_none()
            && !skip
            && rename.is_none()
            && prefix_with_name.is_none()
            && field_names.is_none()
            && rename_all.is_none()
            && rename_all_fields.is_none()
    }
}

/// Parse field/variant-level `#[diagnostic::cheadergen::...]` attributes.
///
/// Field-level attributes are read directly from the field's `attrs` during
/// type resolution (not during indexing).
pub struct FieldAnnotation {
    /// Override the C field/variant name.
    pub rename: Option<String>,
    /// Emit the field as a C bitfield with the given width.
    pub bitfield_width: Option<u64>,
}

impl FieldAnnotation {
    /// Parse all cheadergen directives from a list of rustdoc attributes.
    pub fn from_attrs(attrs: &[Attribute]) -> Self {
        let mut result = FieldAnnotation {
            rename: None,
            bitfield_width: None,
        };
        for attr in attrs {
            if let Attribute::Other(s) = attr {
                let Some(rest) = s.strip_prefix(ATTR_PREFIX) else {
                    continue;
                };
                let rest = rest.strip_suffix(']').unwrap_or(rest);

                if let Some(inner) = rest.strip_prefix("rename(") {
                    let inner = inner.strip_suffix(')').unwrap_or(inner);
                    let name = inner.trim_matches('"');
                    result.rename = Some(name.to_owned());
                } else if let Some(inner) = rest.strip_prefix("bitfield(") {
                    let inner = inner.strip_suffix(')').unwrap_or(inner);
                    // The proc-macro emits the width as a u64 literal (e.g. "4u64"),
                    // so strip the type suffix before parsing.
                    let inner = inner.trim().trim_end_matches("u64");
                    if let Ok(width) = inner.parse::<u64>() {
                        result.bitfield_width = Some(width);
                    }
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Smoke test: each rule maps the four canonical identifier shapes to the
    /// expected output. The actual case-conversion is delegated to `heck` —
    /// these tests guard against accidentally wiring a rule to the wrong heck
    /// trait.
    #[test]
    fn apply_each_rule() {
        assert_eq!(RenameRule::PascalCase.apply("my_status"), "MyStatus");
        assert_eq!(RenameRule::CamelCase.apply("MyStatus"), "myStatus");
        assert_eq!(RenameRule::SnakeCase.apply("MyStatus"), "my_status");
        assert_eq!(
            RenameRule::ScreamingSnakeCase.apply("MyStatus"),
            "MY_STATUS"
        );
    }

    #[test]
    fn from_diagnostic_str_round_trips() {
        for (literal, rule) in [
            ("PascalCase", RenameRule::PascalCase),
            ("camelCase", RenameRule::CamelCase),
            ("snake_case", RenameRule::SnakeCase),
            ("SCREAMING_SNAKE_CASE", RenameRule::ScreamingSnakeCase),
        ] {
            assert_eq!(RenameRule::from_diagnostic_str(literal), Some(rule));
        }
        assert_eq!(RenameRule::from_diagnostic_str("kebab-case"), None);
    }
}
