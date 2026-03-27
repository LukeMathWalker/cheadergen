//! `#[cheadergen::config(field_names(...))]` assigns custom C field names
//! to a tuple struct's positional fields instead of the default `m0`, `m1`, etc.

/// Tuple struct with custom field names.
#[cheadergen::config(export, field_names(x, y))]
#[repr(C)]
pub struct Point2D(pub f64, pub f64);

/// Tuple struct with default naming (no annotation).
#[repr(C)]
pub struct Pair(pub u32, pub u32);

#[unsafe(no_mangle)]
pub extern "C" fn make_pair(a: u32, b: u32) -> Pair {
    Pair(a, b)
}
