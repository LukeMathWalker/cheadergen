//! Verifies that the global `usize_is_size_t = true` flag in
//! `cheadergen.toml` translates `usize`/`isize` to `size_t`/`ptrdiff_t`
//! at every site: function signatures, struct fields, statics, fieldless
//! enum reprs, and type aliases.

#[repr(C)]
pub struct Buffer {
    pub data: *mut u8,
    pub len: usize,
    pub offset: isize,
}

#[repr(usize)]
pub enum BigEnum {
    A,
    B,
}

#[repr(isize)]
pub enum SignedEnum {
    Neg = -1,
    Zero = 0,
}

pub type ItemCount = usize;
pub type Offset = isize;

#[unsafe(no_mangle)]
pub static MAX_LEN: usize = 1024;

#[unsafe(no_mangle)]
pub extern "C" fn make_buffer(len: usize, offset: isize) -> Buffer {
    Buffer {
        data: core::ptr::null_mut(),
        len,
        offset,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn buffer_size(buf: Buffer) -> ItemCount {
    buf.len
}

#[unsafe(no_mangle)]
pub extern "C" fn buffer_offset(buf: Buffer) -> Offset {
    buf.offset
}

#[unsafe(no_mangle)]
pub extern "C" fn classify_big(big: BigEnum, signed_enum: SignedEnum) -> usize {
    let _ = (big, signed_enum);
    0
}
