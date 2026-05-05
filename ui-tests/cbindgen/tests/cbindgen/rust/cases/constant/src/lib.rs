/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[cheadergen::config(export)]
pub const FOO: usize = 10;
#[cheadergen::config(export)]
pub const BAR: &'static str = "hello world";
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
#[cheadergen::config(export)]
pub const HEART: char = '❤';
#[cheadergen::config(export)]
pub const EQUID: char = '𐂃';
#[cheadergen::config(export)]
pub const ZOM: f32 = 3.14;

pub(crate) const DONT_EXPORT_CRATE: i32 = 20;
const DONT_EXPORT_PRIV: i32 = 30;

/// A single-line doc comment.
#[cheadergen::config(export)]
pub const POS_ONE: i8 = 1;
/// A
/// multi-line
/// doc
/// comment.
#[cheadergen::config(export)]
pub const NEG_ONE: i8 = -1;

// Some doc for shifting //
#[cheadergen::config(export)]
pub const SHIFT: i64 = 3;
#[cheadergen::config(export)]
pub const XBOOL: i64 = 1;
#[cheadergen::config(export)]
pub const XFALSE: i64 = (0 << SHIFT) | XBOOL;
#[cheadergen::config(export)]
pub const XTRUE: i64 = 1 << (SHIFT | XBOOL);

#[cheadergen::config(export)]
pub const CAST: u8 = 'A' as u8;
#[cheadergen::config(export)]
pub const DOUBLE_CAST: u32 = 1 as f32 as u32;

#[repr(C)]
struct Foo {
    x: [i32; FOO],
}

#[unsafe(no_mangle)]
pub extern "C" fn root(x: Foo) {}
