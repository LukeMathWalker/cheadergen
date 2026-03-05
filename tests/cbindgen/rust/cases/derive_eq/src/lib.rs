/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[repr(C)]
pub struct Foo {
    a: bool,
    b: i32,
}

#[repr(u8)]
pub enum Bar {
    Baz,
    Bazz { named: Foo },
    FooNamed { different: i32, fields: u32 },
    FooParen(i32, Foo),
}

#[no_mangle]
pub extern "C" fn root(bar: Bar) -> Foo {
    unimplemented!();
}
