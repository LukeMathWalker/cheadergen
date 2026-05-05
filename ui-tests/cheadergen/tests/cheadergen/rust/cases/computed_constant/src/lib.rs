//! Constants whose `expr` is a computed expression (rather than a literal)
//! cannot be lowered to C. Opting in via `#[cheadergen::config(export)]`
//! turns this into a generation error.

const fn make_char() -> char {
    'x'
}
/// Computed char — `is_literal = false`, so this is a hard error under opt-in.
#[cheadergen::config(export)]
pub const COMPUTED_CHAR: char = make_char();

const fn make_str() -> &'static str {
    "computed"
}
/// Computed string — same as above.
#[cheadergen::config(export)]
pub const COMPUTED_STR: &str = make_str();

const fn make_num() -> i32 {
    42
}
/// Computed number — still resolvable because rustdoc evaluates numeric
/// const fn results, so this constant on its own would succeed.
#[cheadergen::config(export)]
pub const COMPUTED_NUM: i32 = make_num();

#[unsafe(no_mangle)]
pub extern "C" fn anchor() {}
