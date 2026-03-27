//! `#[cheadergen::config(skip)]` on an `extern "C"` function excludes it
//! from the generated header. Other functions remain.
//! Types only reachable through skipped functions should also be absent.

#[repr(C)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// Only used by the skipped function — should NOT appear in the header.
#[repr(C)]
pub struct InternalData {
    pub value: u32,
    pub flags: u32,
}

/// This function should NOT appear in the header.
#[cheadergen::config(skip)]
#[unsafe(no_mangle)]
pub extern "C" fn internal_helper(data: &InternalData) -> u32 {
    data.value
}

/// This function SHOULD appear in the header.
#[unsafe(no_mangle)]
pub extern "C" fn make_point(x: f64, y: f64) -> Point {
    Point { x, y }
}
