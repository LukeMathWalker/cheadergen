//! Const generic `#[repr(C)]` structs must be monomorphized into concrete C
//! struct definitions. `Size<4>` and `Size<8>` produce separate typedefs with
//! the const value embedded in the name.

#[repr(C)]
pub struct Size<const N: usize> {
    pub bytes: [u8; N],
}

#[unsafe(no_mangle)]
pub extern "C" fn use_size_4(s: Size<4>) -> Size<4> {
    s
}

#[unsafe(no_mangle)]
pub extern "C" fn use_size_8(s: Size<8>) -> Size<8> {
    s
}
