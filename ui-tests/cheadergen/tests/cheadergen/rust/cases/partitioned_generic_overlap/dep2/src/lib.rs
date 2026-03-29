//! Leaf crate: provides the generic Wrapper type.

#[repr(C)]
pub struct Wrapper<T> {
    pub value: T,
}
