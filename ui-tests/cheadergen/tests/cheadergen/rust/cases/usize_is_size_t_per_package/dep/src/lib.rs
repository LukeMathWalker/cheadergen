//! Dependency for the `usize_is_size_t_per_package` test case. Its types
//! should render with `size_t`/`ptrdiff_t` because of the
//! `[package."size-t-dep"]` override in the entrypoint's `cheadergen.toml`.

#[repr(C)]
pub struct DepBuffer {
    pub len: usize,
    pub offset: isize,
}
