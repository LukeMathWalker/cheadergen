/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[repr(C)]
struct A<'a> {
    data: &'a i32
}

#[repr(C)]
enum E<'a> {
    V,
    U(&'a u8),
}

#[no_mangle]
pub extern "C" fn root<'a>(_a: A<'a>, _e: E<'a>)
{ }
