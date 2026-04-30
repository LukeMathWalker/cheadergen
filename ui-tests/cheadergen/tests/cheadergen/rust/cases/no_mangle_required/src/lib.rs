//! A `pub extern "C"` function is only an exported symbol if it carries
//! `#[unsafe(no_mangle)]` (or `#[unsafe(export_name = "...")]`). Without
//! one of those attributes, Rust mangles the symbol and the function is
//! not callable from C under its source name.
//!
//! cheadergen must therefore only emit a declaration for the function
//! marked with `#[unsafe(no_mangle)]`. The unmangled function below
//! must not appear in the generated header.

#[unsafe(no_mangle)]
pub extern "C" fn exported_fn(x: i32) -> i32 {
    x + 1
}

pub extern "C" fn mangled_fn(x: i32) -> i32 {
    x + 2
}
