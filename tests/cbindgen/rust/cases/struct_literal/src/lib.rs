/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[repr(C)]
struct Foo {
    a: i32,
    b: u32,
}

struct Bar {
    a: i32,
    b: u32,
}

impl Foo {
    pub const FOO: Foo = Foo { a: 42, b: 47, };
    pub const FOO2: Self = Foo { a: 42, b: 47, };
    pub const FOO3: Self = Self { a: 42, b: 47, };
    pub const BAZ: Bar = Bar { a: 42, b: 47, };
}

pub const BAR: Foo = Foo { a: 42, b: 1337, };
pub const BAZZ: Bar = Bar { a: 42, b: 1337, };

#[no_mangle]
pub extern "C" fn root(x: Foo, bar: Bar) { }
