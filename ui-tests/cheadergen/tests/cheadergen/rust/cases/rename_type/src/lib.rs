//! `#[cheadergen::config(rename = "...")]` overrides the C type name.

/// Struct renamed from `InternalPoint` to `Point` in the C header.
#[cheadergen::config(rename = "Point")]
#[repr(C)]
pub struct InternalPoint {
    pub x: f64,
    pub y: f64,
}

/// Enum renamed in the C header.
#[cheadergen::config(rename = "Status")]
#[repr(C)]
pub enum InternalStatus {
    Ok,
    Error,
}

#[unsafe(no_mangle)]
pub extern "C" fn make_point(x: f64, y: f64) -> InternalPoint {
    InternalPoint { x, y }
}

#[unsafe(no_mangle)]
pub extern "C" fn get_status() -> InternalStatus {
    InternalStatus::Ok
}
