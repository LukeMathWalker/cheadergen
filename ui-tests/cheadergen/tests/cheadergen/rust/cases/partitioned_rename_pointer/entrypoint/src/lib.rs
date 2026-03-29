//! Target crate: uses renamed type behind pointer only (forward-decl with renamed name).

#[unsafe(no_mangle)]
pub extern "C" fn use_data(ptr: *const partitioned_rename_pointer_dep::InternalData) -> *const partitioned_rename_pointer_dep::InternalData {
    ptr
}
