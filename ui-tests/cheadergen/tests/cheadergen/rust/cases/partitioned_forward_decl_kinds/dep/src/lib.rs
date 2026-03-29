//! Dep crate: defines struct, union, and fieldless enum for forward-decl testing.

#[repr(C)]
pub struct MyStruct {
    pub a: i32,
}

#[repr(C)]
pub union MyUnion {
    pub x: i32,
    pub y: f32,
}

#[repr(C)]
pub enum MyEnum {
    A,
    B,
    C,
}
