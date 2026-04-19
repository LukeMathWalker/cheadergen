//! Same struct reached via two distinct lifetimes (`'static` and `'a`).
//! Must produce a single C definition — lifetimes have no C equivalent.

#[repr(C)]
pub struct Foo<'a> {
    pub data: &'a i32,
}

#[unsafe(no_mangle)]
pub extern "C" fn use_foo_static(f: Foo<'static>) -> Foo<'static> {
    f
}

#[unsafe(no_mangle)]
pub extern "C" fn use_foo_borrowed<'a>(f: Foo<'a>) -> Foo<'a> {
    f
}
