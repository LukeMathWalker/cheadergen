//! Dep crate: defines struct, union, fieldless enum, type alias, and
//! transparent wrapper for forward-decl testing.

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

/// Type alias to a `repr(C)` struct — emitted as a `typedef` in C.
pub type MyAlias = MyStruct;

/// `repr(transparent)` wrapper around a `repr(C)` struct — emitted as a
/// `typedef` in C.
#[repr(transparent)]
pub struct MyTransparent(pub MyStruct);
