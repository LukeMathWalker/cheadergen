//! `#[repr(align(N))]` without `#[repr(C)]` is rejected with an error:
//! without a defined layout we can't honor the alignment intent.

#[repr(align(16))]
pub struct BareAligned {
    pub value: u64,
}

#[unsafe(no_mangle)]
pub extern "C" fn bare_aligned(v: BareAligned) -> u64 {
    v.value
}
