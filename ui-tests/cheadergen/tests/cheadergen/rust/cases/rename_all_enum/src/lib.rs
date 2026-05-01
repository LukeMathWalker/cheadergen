//! `#[cheadergen::config(rename_all = "...")]` on an enum bulk-renames each
//! variant using the casing rule.

#[cheadergen::config(export, rename_all = "SCREAMING_SNAKE_CASE")]
#[repr(C)]
pub enum Status {
    Ok,
    ErrorState,
    Pending,
}
