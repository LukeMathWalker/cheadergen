/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[repr(C)]
struct Foo {
    a: i32,
    b: u32,
    bar: Bar,
}

#[repr(C)]
struct Bar {
    a: i32,
}

pub const VAL: Foo = Foo {
    a: 42,
    b: 1337,
    bar: Bar { a: 323 },
};

#[no_mangle]
pub extern "C" fn root(x: Foo) {}
