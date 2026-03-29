//! Target crate: uses opaque dep type behind a pointer.

#[repr(C)]
pub struct LocalStruct {
    pub a: i32,
}

#[unsafe(no_mangle)]
pub extern "C" fn use_dep(ptr: *const partitioned_opaque_unchanged_dep::DepStruct) -> *const partitioned_opaque_unchanged_dep::DepStruct {
    ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn use_local(s: LocalStruct) -> LocalStruct {
    s
}
