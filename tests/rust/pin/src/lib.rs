/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[repr(C)]
struct PinTest {
    pinned_box: Pin<Box<i32>>,
    pinned_ref: Pin<&mut i32>
}

#[no_mangle]
pub extern "C" fn root(s: Pin<&mut i32>, p: PinTest) {}
