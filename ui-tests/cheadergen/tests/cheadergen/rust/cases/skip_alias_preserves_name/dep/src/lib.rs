//! Dependency holding a type alias whose name should survive through to the
//! C output rather than being expanded to the underlying primitive.

#[allow(non_camel_case_types)]
pub type size_t = usize;
