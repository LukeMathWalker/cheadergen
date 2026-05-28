use std::ptr::NonNull;

#[cheadergen::config(export, field_names(ptr))]
#[repr(C)]
pub struct Tuple(#[cheadergen(const_ptr)] NonNull<u32>);

#[cheadergen::config(export)]
#[repr(C)]
pub struct Struct {
    #[cheadergen(const_ptr)]
    pub ptr: NonNull<u32>,
}

#[cheadergen::config(export)]
#[repr(C)]
pub union Union {
    #[cheadergen(const_ptr)]
    pub ptr: NonNull<u32>,
}

fn main() {}
