//! Free-standing constants are now opt-in: a `pub const` without
//! `#[cheadergen::config(export)]` does not appear in the generated header.

pub const FOO: u32 = 1;

pub const BAR: &str = "ignored";

#[unsafe(no_mangle)]
pub extern "C" fn anchor() -> u32 {
    FOO
}
