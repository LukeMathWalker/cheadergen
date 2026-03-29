//! Target crate: uses dep type only behind a pointer.

#[unsafe(no_mangle)]
pub extern "C" fn use_dep(ptr: *const partitioned_pointer_only_dep::DepStruct) -> *const partitioned_pointer_only_dep::DepStruct {
    ptr
}
