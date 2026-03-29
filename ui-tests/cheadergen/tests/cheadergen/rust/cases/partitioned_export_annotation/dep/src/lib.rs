//! Dep crate: exports a type via annotation without it being used in any function.

#[repr(C)]
#[cheadergen::config(export)]
pub struct ExportedConfig {
    pub width: i32,
    pub height: i32,
}
