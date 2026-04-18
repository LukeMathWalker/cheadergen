//! Target for the `partitioned_header_rename_bundle` test.
//!
//! `header_name` must be rejected when `bundle = true`.

#[unsafe(no_mangle)]
pub extern "C" fn simple() -> i32 {
    42
}
