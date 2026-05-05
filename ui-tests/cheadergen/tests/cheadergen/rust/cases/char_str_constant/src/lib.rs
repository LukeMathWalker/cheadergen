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

#[cheadergen::config(export)]
pub const GREETING: &str = "hello world";

#[unsafe(no_mangle)]
pub extern "C" fn anchor() {}
