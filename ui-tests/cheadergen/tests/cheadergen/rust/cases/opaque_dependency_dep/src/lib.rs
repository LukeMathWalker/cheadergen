//! Helper crate for the `opaque_dependency` test case.

#[repr(C)]
pub struct DepStruct {
    pub x: u32,
    pub y: f64,
}
