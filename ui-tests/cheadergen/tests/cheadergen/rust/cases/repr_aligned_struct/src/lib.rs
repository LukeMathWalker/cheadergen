//! `#[repr(C, align(N))]` on a struct emits the `CHEADERGEN_ALIGNED(N)` macro
//! and forces the tagged struct form.

#[repr(C, align(16))]
pub struct AlignedPoint {
    pub x: f64,
    pub y: f64,
}

#[unsafe(no_mangle)]
pub extern "C" fn aligned_point_new(x: f64, y: f64) -> AlignedPoint {
    AlignedPoint { x, y }
}
