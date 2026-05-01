//! Per-variant `#[cheadergen(rename = "...")]` always wins over the bulk
//! `rename_all` rule: the explicit name is used verbatim and the casing rule
//! is skipped for that variant.

#[cheadergen::config(export, rename_all = "SCREAMING_SNAKE_CASE")]
#[repr(C)]
pub enum Color {
    Red,
    /// Explicit override wins — emitted as `explicit_green`, not `GREEN`.
    #[cheadergen(rename = "explicit_green")]
    Green,
    Blue,
}
