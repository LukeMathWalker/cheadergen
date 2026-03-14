//! Plain `#[repr(C)]` unions must be emitted as C `union` definitions.
//!
//! Tests: basic union, union with ZST fields (should be skipped),
//! non-repr(C) union (should be opaque behind pointer).

use std::marker::PhantomData;

/// A non-repr(C) union — should be emitted as an opaque forward declaration
/// when only used behind a pointer.
union Opaque {
    x: i32,
    y: f32,
}

/// A basic repr(C) union with primitive fields.
#[repr(C)]
pub union Normal {
    pub x: i32,
    pub y: f32,
}

/// A repr(C) union where ZST fields (`()`, `PhantomData`) should be skipped.
#[repr(C)]
pub union WithZST {
    pub x: i32,
    pub y: f32,
    pub z: (),
    pub w: PhantomData<i32>,
}

#[unsafe(no_mangle)]
pub extern "C" fn root(a: *mut Opaque, b: Normal, c: WithZST) {}
