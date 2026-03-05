/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[no_mangle]
pub static NUMBER: i32 = 10;

#[repr(C)]
struct Foo {
}

struct Bar {
}

#[no_mangle]
pub static mut FOO: Foo = Foo { };
#[no_mangle]
pub static BAR: Bar = Bar { };

#[no_mangle]
pub extern "C" fn root() { }
