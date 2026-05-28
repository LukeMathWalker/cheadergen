//! `#[cheadergen(const_ptr)]` is rejected on fields that do not lower to pointers.

#[cheadergen::config(export)]
#[repr(C)]
pub struct Config {
    #[cheadergen(const_ptr)]
    pub value: u32,
}
