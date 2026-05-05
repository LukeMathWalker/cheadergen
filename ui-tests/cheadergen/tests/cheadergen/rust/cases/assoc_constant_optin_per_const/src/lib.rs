//! Per-constant opt-in for assoc constants: only the ones carrying
//! `#[cheadergen::config(export)]` reach the header. Sibling constants
//! without the annotation are silently dropped.

#[repr(C)]
pub struct Foo {
    pub field: u32,
}

impl Foo {
    #[cheadergen::config(export)]
    pub const KEPT: u32 = 1;

    pub const DROPPED: u32 = 2;

    #[cheadergen::config(export)]
    pub const ALSO_KEPT: &'static str = "hi";
}

#[unsafe(no_mangle)]
pub extern "C" fn anchor(x: Foo) -> u32 {
    x.field
}
