//! `#[cheadergen::config(export)]` is a hard error on a non-`pub` constant
//! (free-standing or associated). Only fully-public items can leak into the
//! C header.

#[cheadergen::config(export)]
pub(crate) const PRIVATE_FREE: i32 = 1;

#[cheadergen::config(export)]
const TRULY_PRIVATE_FREE: i32 = 2;

#[repr(C)]
pub struct Foo {
    pub field: u32,
}

impl Foo {
    #[cheadergen::config(export)]
    pub(crate) const PRIVATE_ASSOC: i32 = 3;

    #[cheadergen::config(export)]
    const TRULY_PRIVATE_ASSOC: i32 = 4;
}

#[unsafe(no_mangle)]
pub extern "C" fn anchor(x: Foo) -> u32 {
    x.field
}
