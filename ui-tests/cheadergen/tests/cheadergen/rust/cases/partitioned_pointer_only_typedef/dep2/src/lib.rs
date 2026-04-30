//! Holds both a typedef and a real struct. The struct is referenced
//! by-value from `dep::Outer`, so this crate keeps its own header. The
//! entrypoint references the typedef only behind a pointer — its header
//! should declare the typedef inline, not `#include` this crate's header.

#[repr(C)]
pub struct Inner {
    pub x: i32,
}

pub type Aliased = u32;
