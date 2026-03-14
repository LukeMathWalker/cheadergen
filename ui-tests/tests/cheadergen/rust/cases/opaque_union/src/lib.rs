//! Opaque unions (non-repr(C)) must emit `union` forward declarations,
//! not `struct`.

/// A non-repr(C) union — opaque, should use `union` tag.
pub union NonReprC {
    pub x: i32,
    pub y: f32,
}

#[unsafe(no_mangle)]
pub extern "C" fn use_non_repr_c(a: *mut NonReprC) {}
