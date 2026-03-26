#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VariantStatus {
    /// cheadergen succeeds but produces different output than the cbindgen expectation.
    HeaderDiff,
    /// cheadergen fails to generate output (non-zero exit).
    GenerationFails,
    Skip,
    Exclude,
}
