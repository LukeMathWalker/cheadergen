/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[cheadergen::config(export)]
pub const CONSTANT_I64: i64 = 216;
#[cheadergen::config(export)]
pub const CONSTANT_FLOAT32: f32 = 312.292;
#[cheadergen::config(export)]
pub const DELIMITER: char = ':';
#[cheadergen::config(export)]
pub const LEFTCURLY: char = '{';
#[repr(C)]
struct Foo {
    x: i32,
}

#[expect(non_upper_case_globals)]
pub const SomeFoo: Foo = Foo { x: 99, };

impl Foo {
    #[cheadergen::config(export)]
    pub const CONSTANT_I64_BODY: i64 = 216;
}
