//! `#[cheadergen::config(skip)]` on a `#[no_mangle]` static excludes it
//! from the generated header. Other statics remain.

/// This static should NOT appear in the header.
#[cheadergen::config(skip)]
#[unsafe(no_mangle)]
pub static INTERNAL_VERSION: u32 = 42;

/// This static SHOULD appear in the header.
#[unsafe(no_mangle)]
pub static PUBLIC_VERSION: u32 = 1;
