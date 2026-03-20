//! A struct without `#[repr(C)]` used by-value in an extern "C" function.
//!
//! cheadergen should emit a warning diagnostic and fall back to an opaque
//! forward declaration for the type.

pub struct Opaque {
    pub x: f64,
    pub y: f64,
}

#[repr(C)]
pub struct Visible {
    pub a: u32,
    pub b: u32,
}

#[unsafe(no_mangle)]
pub extern "C" fn takes_opaque(v: Opaque) -> Visible {
    let _ = v;
    Visible { a: 1, b: 2 }
}
