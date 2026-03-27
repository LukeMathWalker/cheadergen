//! Tests that `[package.skip-dep]` with `types = "skip"` suppresses all emission
//! for dependency types. The user is expected to provide definitions via an
//! included header.

#[repr(C)]
pub struct LocalStruct {
    pub a: i32,
    pub b: i32,
}

#[unsafe(no_mangle)]
pub extern "C" fn use_skip_dep(ptr: *const skip_dep::SkipStruct) -> *const skip_dep::SkipStruct {
    ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn use_local(s: LocalStruct) -> LocalStruct {
    s
}
