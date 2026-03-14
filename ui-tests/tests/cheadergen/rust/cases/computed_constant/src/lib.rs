//! Constants from computed expressions — these should be skipped by cheadergen.

const fn make_char() -> char {
    'x'
}
/// Computed char — skipped because is_literal = false.
pub const COMPUTED_CHAR: char = make_char();

const fn make_str() -> &'static str {
    "computed"
}
/// Computed string — skipped because is_literal = false.
pub const COMPUTED_STR: &str = make_str();

const fn make_num() -> i32 {
    42
}
/// Computed number — still emitted because rustdoc evaluates numeric values.
pub const COMPUTED_NUM: i32 = make_num();

#[unsafe(no_mangle)]
pub extern "C" fn anchor() {}
