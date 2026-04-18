//! Library crate for the `bin_only_skipped` test — the companion to the
//! binary crate, which cheadergen should skip with a warning.

#[unsafe(no_mangle)]
pub extern "C" fn bin_only_skipped_answer() -> i32 {
    42
}
