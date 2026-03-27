//! `#[cheadergen(bitfield = N)]` emits a C bitfield with the specified width.

#[cheadergen::config(export)]
#[repr(C)]
pub struct Flags {
    #[cheadergen(bitfield = 4)]
    pub low_nibble: u8,
    #[cheadergen(bitfield = 4)]
    pub high_nibble: u8,
    /// A regular field (no bitfield).
    pub value: u32,
}
