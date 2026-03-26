/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// cbindgen:rename-associated-constant=UpperCase
#[repr(C)]
struct Foo {}

impl Foo {
    pub const GA: i32 = 10;
    pub const ZO: f32 = 3.14;
}

#[unsafe(no_mangle)]
pub extern "C" fn root(x: Foo) { }
