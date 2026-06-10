//! Char and string constants emitted as `#define` macros.
//!
//! Each constant must opt in with `#[cheadergen::config(export)]`.

/// ASCII delimiter.
#[cheadergen::config(export)]
pub const DELIMITER: char = ':';
#[cheadergen::config(export)]
pub const LEFTCURLY: char = '{';
#[cheadergen::config(export)]
pub const QUOTE: char = '\'';
#[cheadergen::config(export)]
pub const TAB: char = '\t';
#[cheadergen::config(export)]
pub const NEWLINE: char = '\n';
/// A Unicode heart.
#[cheadergen::config(export)]
pub const HEART: char = '❤';
#[cheadergen::config(export)]
pub const EQUID: char = '𐂃';
/// A Unicode escape — C has no braced `\u{…}` form.
#[cheadergen::config(export)]
pub const ESCAPED_HEART: char = '\u{2764}';
/// A Unicode escape that lands in the ASCII range.
#[cheadergen::config(export)]
pub const ESCAPED_AT: char = '\u{40}';
/// A hex escape, valid in both Rust and C.
#[cheadergen::config(export)]
pub const ESCAPE_CHAR: char = '\x1B';

#[cheadergen::config(export)]
pub const GREETING: &str = "hello world";

#[unsafe(no_mangle)]
pub extern "C" fn anchor() {}
