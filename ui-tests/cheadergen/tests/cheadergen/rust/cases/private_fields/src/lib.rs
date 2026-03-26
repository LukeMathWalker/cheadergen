//! Types with known repr but private fields should emit full definitions,
//! not opaque forward declarations.

/// A repr(C) struct with private fields.
#[repr(C)]
pub struct PrivateStruct {
    x: i32,
    y: f32,
}

/// A repr(C) union with private fields.
#[repr(C)]
pub union PrivateUnion {
    x: i32,
    y: f32,
}

/// A repr(C) enum with a private-field variant.
#[repr(C)]
pub enum PrivateEnum {
    A { x: i32, y: f32 },
    B,
}

#[unsafe(no_mangle)]
pub extern "C" fn use_struct(a: PrivateStruct) -> PrivateStruct {
    a
}

#[unsafe(no_mangle)]
pub extern "C" fn use_union(a: PrivateUnion) -> PrivateUnion {
    a
}

#[unsafe(no_mangle)]
pub extern "C" fn use_enum(a: PrivateEnum) -> PrivateEnum {
    a
}
