//! Verifies that `[package."size-t-dep"].usize_is_size_t = true` flips the
//! translation only for items defined in `size-t-dep`, while items defined in
//! the root crate continue to use the default `uintptr_t`/`intptr_t`.

#[repr(C)]
pub struct LocalBuffer {
    pub len: usize,
    pub offset: isize,
}

#[unsafe(no_mangle)]
pub extern "C" fn use_dep(b: size_t_dep::DepBuffer) -> size_t_dep::DepBuffer {
    b
}

#[unsafe(no_mangle)]
pub extern "C" fn use_local(b: LocalBuffer) -> usize {
    b.len
}
