/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[repr(C)]
pub struct Foo {
    field: u32,
}

impl Foo {
    pub const FIELD_RELATED_CONSTANT: u32 = 0;
}

#[repr(C)]
pub struct Bar {
    field: u32,
}

impl Bar {
    pub const FIELD_RELATED_CONSTANT: u32 = 0;
}

#[no_mangle]
pub extern "C" fn root(a: Foo, b: Bar) {}
