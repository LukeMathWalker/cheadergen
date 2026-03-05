/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

struct A {
    x: i32,
    y: f32,
}

#[repr(C)]
struct B {
    x: i32,
    y: f32,
}

#[no_mangle]
pub extern "C" fn root(a: *const A, b: B) {}
