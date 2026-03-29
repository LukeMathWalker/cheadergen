//! Target crate: uses tagged union from dep behind a pointer only.

#[unsafe(no_mangle)]
pub extern "C" fn use_message(ptr: *const partitioned_tagged_union_fwd_dep::Message) -> *const partitioned_tagged_union_fwd_dep::Message {
    ptr
}
