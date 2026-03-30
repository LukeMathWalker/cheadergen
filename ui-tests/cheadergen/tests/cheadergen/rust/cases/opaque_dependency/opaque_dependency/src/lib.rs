//! Tests that `[package.opaque-dep]` with `types = "opaque"` forces dependency types
//! to be emitted as forward declarations only, even when used behind a pointer.

#[repr(C)]
pub struct LocalStruct {
    pub a: i32,
    pub b: i32,
}

#[unsafe(no_mangle)]
pub extern "C" fn use_dep(ptr: *const opaque_dep::DepStruct) -> *const opaque_dep::DepStruct {
    ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn use_local(s: LocalStruct) -> LocalStruct {
    s
}
