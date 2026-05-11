//! `#[repr(C, packed(N))]` is rejected — packing isn't supported yet, even
//! when combined with `repr(C)`.

#[repr(C, packed(4))]
pub struct PackedN {
    pub a: u8,
    pub b: u64,
}

#[unsafe(no_mangle)]
pub extern "C" fn packed_n_new() -> PackedN {
    PackedN { a: 0, b: 0 }
}
