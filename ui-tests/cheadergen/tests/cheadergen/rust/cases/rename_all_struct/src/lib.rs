//! `#[cheadergen::config(rename_all = "...")]` on a struct bulk-renames
//! every field name using the casing rule. A per-field
//! `#[cheadergen(rename = "...")]` overrides the bulk rule.

#[cheadergen::config(export, rename_all = "camelCase")]
#[repr(C)]
pub struct Settings {
    pub max_value: u32,
    pub min_value: u32,
    /// Per-field rename wins over the bulk rule.
    #[cheadergen(rename = "explicitName")]
    pub other_field: u32,
}
