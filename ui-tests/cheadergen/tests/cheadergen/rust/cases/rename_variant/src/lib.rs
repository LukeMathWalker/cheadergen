//! `#[cheadergen(rename = "...")]` on enum variants overrides the C variant name.

#[cheadergen::config(export)]
#[repr(C)]
pub enum Color {
    Red,
    #[cheadergen(rename = "COLOR_GREEN")]
    Green,
    Blue,
}
