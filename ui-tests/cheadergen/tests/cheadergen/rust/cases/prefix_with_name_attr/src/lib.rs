//! `#[cheadergen::config(prefix_with_name)]` prefixes enum variant names
//! with the enum name. `prefix_with_name = false` disables it even when
//! the global config has it enabled.

/// Variants should be prefixed: `Status_Ok`, `Status_Error`, `Status_Pending`.
#[cheadergen::config(export, prefix_with_name)]
#[repr(C)]
pub enum Status {
    Ok,
    Error,
    Pending,
}

/// Variants should NOT be prefixed (explicit false).
#[cheadergen::config(export, prefix_with_name = false)]
#[repr(C)]
pub enum Flags {
    A,
    B,
    C,
}

/// No annotation — uses global default (which is `false` by default).
#[repr(C)]
pub enum Color {
    Red,
    Green,
    Blue,
}

#[unsafe(no_mangle)]
pub extern "C" fn get_color() -> Color {
    Color::Red
}
