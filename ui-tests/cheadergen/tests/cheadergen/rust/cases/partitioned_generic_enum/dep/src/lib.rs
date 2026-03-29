//! Dependency crate providing a generic tagged union.

#[repr(C)]
pub enum Either<T> {
    Value(T),
    None,
}
