/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[repr(C)]
pub struct Foo<T> {
    something: *const i32,
    phantom: std::marker::PhantomData<T>,
}

#[repr(C)]
pub union Bar {
    something: i32,
    subexpressions: Foo<Self>,
}

#[no_mangle]
pub extern "C" fn root(b: Bar) {}
