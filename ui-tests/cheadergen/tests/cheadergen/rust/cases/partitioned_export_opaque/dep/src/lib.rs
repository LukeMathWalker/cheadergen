//! Dep crate: exports a type as opaque via annotation.

#[repr(C)]
#[cheadergen::config(export, opaque)]
pub struct OpaqueConfig {
    pub internal: i32,
}
