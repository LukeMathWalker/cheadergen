//! Types annotated with `#[cheadergen::export]` for forced header inclusion.

use cheadergen_macros as cheadergen;

/// A config struct exported via annotation, not referenced by any FFI function.
#[cheadergen::export]
#[repr(C)]
pub struct Config {
    pub width: u32,
    pub height: u32,
}

/// A status enum exported via annotation, not referenced by any FFI function.
#[cheadergen::export]
#[repr(C)]
pub enum Status {
    Ok,
    Error,
    Pending,
}

/// A struct that is both annotated AND referenced by an FFI function.
#[cheadergen::export]
#[repr(C)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// A type without `#[repr(C)]` — should produce an opaque forward declaration.
#[cheadergen::export]
pub struct OpaqueHandle {
    _inner: u64,
}

#[unsafe(no_mangle)]
pub extern "C" fn make_point(x: f64, y: f64) -> Point {
    Point { x, y }
}
