//! Defines a union, a `repr(C)` data-bearing enum, a `repr(u8)` data-bearing
//! enum, and a fieldless enum. Each is referenced by-value through an
//! extern fn so the dep keeps its own header. The entrypoint references
//! each only behind a pointer.

#[repr(C)]
pub union FooUnion {
    pub i: i32,
    pub f: f32,
}

#[repr(C)]
pub enum TaggedC {
    A(i32),
    B,
}

#[repr(u8)]
pub enum TaggedInt {
    X(u32),
    Y,
}

#[repr(C)]
pub enum Fieldless {
    One,
    Two,
}

#[unsafe(no_mangle)]
pub extern "C" fn make_union() -> FooUnion {
    FooUnion { i: 0 }
}

#[unsafe(no_mangle)]
pub extern "C" fn make_tagged_c() -> TaggedC {
    TaggedC::B
}

#[unsafe(no_mangle)]
pub extern "C" fn make_tagged_int() -> TaggedInt {
    TaggedInt::Y
}

#[unsafe(no_mangle)]
pub extern "C" fn make_fieldless() -> Fieldless {
    Fieldless::One
}
