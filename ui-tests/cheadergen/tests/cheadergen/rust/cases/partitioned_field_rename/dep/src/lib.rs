//! Dep crate: tuple struct with custom field names via annotation.

#[repr(C)]
#[cheadergen::config(export, field_names(x, y))]
pub struct Vec2(pub f32, pub f32);
