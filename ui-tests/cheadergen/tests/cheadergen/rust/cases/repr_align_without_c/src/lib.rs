//! `#[repr(align(N))]` without `#[repr(C)]` is rejected with an error:
//! without a defined layout we can't honor the alignment intent.

#[repr(align(16))]
pub struct BareAligned {
    pub value: u64,
}

#[unsafe(no_mangle)]
#[expect(improper_ctypes_definitions)]
pub extern "C" fn bare_aligned(v: BareAligned) -> u64 {
    v.value
}
