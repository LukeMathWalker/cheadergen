//! `#[cheadergen(const_ptr)]` qualifies pointer fields as const in C.

use std::ptr::NonNull;

#[cheadergen::config(export)]
#[repr(C)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[cheadergen::config(export)]
#[repr(C)]
pub struct StructFields {
    #[cheadergen(const_ptr)]
    pub nonnull: NonNull<Point>,
    #[cheadergen(const_ptr)]
    pub optional_nonnull: Option<NonNull<Point>>,
    #[cheadergen(const_ptr)]
    pub raw_mut: *mut Point,
    #[cheadergen(const_ptr)]
    pub raw_const: *const Point,
    #[cheadergen(const_ptr)]
    pub mutable_ref: &'static mut Point,
    pub still_mut: NonNull<Point>,
}

#[cheadergen::config(export, field_names(ptr))]
#[repr(C)]
pub struct TupleField(#[cheadergen(const_ptr)] NonNull<Point>);

#[cheadergen::config(export)]
#[repr(C)]
pub union UnionField {
    #[cheadergen(const_ptr)]
    pub ptr: NonNull<Point>,
    pub bits: usize,
}

#[cheadergen::config(export)]
#[repr(C)]
pub enum TaggedField {
    WithPtr {
        #[cheadergen(const_ptr)]
        ptr: NonNull<Point>,
    },
    Empty,
}
