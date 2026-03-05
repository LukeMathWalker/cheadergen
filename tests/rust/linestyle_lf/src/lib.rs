/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[repr(C)]
struct Dummy {
    x: i32,
    y: f32,
}

#[no_mangle]
pub extern "C" fn root(d: Dummy) {}
