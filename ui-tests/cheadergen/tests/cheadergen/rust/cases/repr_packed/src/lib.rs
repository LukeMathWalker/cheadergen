//! `#[repr(packed)]` is rejected: cheadergen has no support yet for emitting
//! packed C struct definitions.

#[repr(packed)]
pub struct Packed {
    pub a: u8,
    pub b: u32,
}

#[unsafe(no_mangle)]
pub extern "C" fn packed_new() -> Packed {
    Packed { a: 0, b: 0 }
}
