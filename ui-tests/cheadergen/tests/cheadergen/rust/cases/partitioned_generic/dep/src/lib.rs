//! Dependency crate providing a generic type.

#[repr(C)]
pub struct Wrapper<T> {
    pub value: T,
}
