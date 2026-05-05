/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[cheadergen::config(export)]
pub const FOO: usize = 10;
#[cheadergen::config(export)]
pub const BAR: &'static str = "hello world";
#[cheadergen::config(export)]
pub const ZOM: f32 = 3.14;

#[repr(C)]
struct Foo {
    x: [i32; FOO],
}

#[unsafe(no_mangle)]
pub extern "C" fn root(x: Foo) { }
