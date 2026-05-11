//! Two aligned types in the same header: the `CHEADERGEN_ALIGNED(N)` macro
//! should be emitted exactly once in the prologue.

#[repr(C, align(16))]
pub struct AlignedA {
    pub x: u64,
}

#[repr(C, align(32))]
pub struct AlignedB {
    pub y: u64,
}

#[unsafe(no_mangle)]
pub extern "C" fn use_both(_a: AlignedA, _b: AlignedB) {}
