#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VariantStatus {
    Xfail,
    Skip,
    Exclude,
}
