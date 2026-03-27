//! `#[cheadergen(rename = "...")]` on struct fields overrides the C field name.

#[cheadergen::config(export)]
#[repr(C)]
pub struct Config {
    #[cheadergen(rename = "raw_width")]
    pub width: u32,
    #[cheadergen(rename = "raw_height")]
    pub height: u32,
    /// A field without rename — should use its Rust name.
    pub depth: u32,
}
