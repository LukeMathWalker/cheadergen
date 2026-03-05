/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub const FOO: i32 = 10;
pub const BAR: &'static str = "hello world";
pub const ZOM: f32 = 3.14;

#[repr(C)]
struct Foo {
    x: [i32; FOO],
}

#[no_mangle]
pub extern "C" fn root(x: Foo) { }
