//! Target crate: uses the opaque-exported type behind a pointer.

#[unsafe(no_mangle)]
pub extern "C" fn use_config(ptr: *const partitioned_export_opaque_dep::OpaqueConfig) -> *const partitioned_export_opaque_dep::OpaqueConfig {
    ptr
}
