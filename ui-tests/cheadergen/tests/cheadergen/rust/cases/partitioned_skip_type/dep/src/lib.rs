//! Dep crate: `Visible` is normal, `Hidden` is skipped via annotation.

#[repr(C)]
pub struct Visible {
    pub value: i32,
}

#[repr(C)]
#[cheadergen::config(skip)]
pub struct Hidden {
    pub secret: i32,
}
