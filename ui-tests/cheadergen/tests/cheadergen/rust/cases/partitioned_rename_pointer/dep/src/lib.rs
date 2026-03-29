//! Dep crate: defines `InternalData` renamed to `Data`.

#[repr(C)]
#[cheadergen::config(rename = "Data")]
pub struct InternalData {
    pub value: f64,
}
