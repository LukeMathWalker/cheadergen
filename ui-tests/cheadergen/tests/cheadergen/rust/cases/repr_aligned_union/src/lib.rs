//! `#[repr(C, align(N))]` on a union emits the `CHEADERGEN_ALIGNED(N)` macro.

#[repr(C, align(8))]
pub union AlignedScalar {
    pub as_i64: i64,
    pub as_f64: f64,
}

#[unsafe(no_mangle)]
pub extern "C" fn aligned_scalar_from_i64(value: i64) -> AlignedScalar {
    AlignedScalar { as_i64: value }
}
