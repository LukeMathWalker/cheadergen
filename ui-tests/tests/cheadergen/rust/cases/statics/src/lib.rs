//! Statics: no_mangle, export_name, and unexported.

/// Exported via `#[unsafe(no_mangle)]`.
#[unsafe(no_mangle)]
pub static GLOBAL_COUNT: u32 = 0;

/// Mutable static exported via `#[unsafe(no_mangle)]`.
#[unsafe(no_mangle)]
pub static mut MUTABLE_STATE: i64 = -1;

/// Exported with a custom symbol name via `#[export_name]`.
#[unsafe(export_name = "custom_name")]
pub static RENAMED: f32 = 3.14;

/// Not exported — no `#[no_mangle]` or `#[export_name]`.
pub static INTERNAL_ONLY: u8 = 42;
