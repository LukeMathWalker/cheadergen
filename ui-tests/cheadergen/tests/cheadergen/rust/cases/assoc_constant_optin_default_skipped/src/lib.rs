//! Associated constants are opt-in. Even when the parent type is exported,
//! its inherent `pub const`s do not reach the header unless each one carries
//! `#[cheadergen::config(export)]`.

#[repr(C)]
pub struct Foo {
    pub field: u32,
}

impl Foo {
    pub const ANSWER: u32 = 42;
    pub const NAME: &'static str = "foo";
}

#[unsafe(no_mangle)]
pub extern "C" fn anchor(x: Foo) -> u32 {
    x.field
}
