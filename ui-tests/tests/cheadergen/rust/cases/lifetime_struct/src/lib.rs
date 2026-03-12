//! Structs generic over lifetime parameters. Lifetime parameters must be
//! stripped in C output (C has no concept of lifetimes), and references
//! become pointers. Mixed lifetime + type parameter structs are
//! monomorphized by type parameters only.

#[repr(C)]
pub struct Foo<'a> {
    pub data: &'a i32,
}

#[repr(C)]
pub struct Bar<'a, 'b> {
    pub first: &'a u32,
    pub second: &'b mut f64,
}

#[repr(C)]
pub struct Baz<'a, T> {
    pub value: &'a T,
    pub flag: bool,
}

#[unsafe(no_mangle)]
pub extern "C" fn use_foo(f: Foo<'_>) -> Foo<'_> {
    f
}

#[unsafe(no_mangle)]
pub extern "C" fn use_bar<'a, 'b>(b: Bar<'a, 'b>) -> Bar<'a, 'b> {
    b
}

#[unsafe(no_mangle)]
pub extern "C" fn use_baz_i32(b: Baz<'_, i32>) -> Baz<'_, i32> {
    b
}

#[unsafe(no_mangle)]
pub extern "C" fn use_baz_f64(b: Baz<'_, f64>) -> Baz<'_, f64> {
    b
}
