//! Char and string constants emitted as `#define` macros.

/// ASCII delimiter.
pub const DELIMITER: char = ':';
pub const LEFTCURLY: char = '{';
pub const QUOTE: char = '\'';
pub const TAB: char = '\t';
pub const NEWLINE: char = '\n';
/// A Unicode heart.
pub const HEART: char = '❤';
pub const EQUID: char = '𐂃';

pub const GREETING: &str = "hello world";

#[unsafe(no_mangle)]
pub extern "C" fn anchor() {}
