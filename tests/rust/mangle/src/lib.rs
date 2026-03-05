/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[repr(C)]
pub struct Foo<T> {
    a: T,
}

pub type Boo = Foo<u8>;

/// cbindgen:prefix-with-name=true
#[repr(C)]
pub enum Bar {
    Some,
    Thing,
}

#[no_mangle]
pub extern "C" fn root(
    x: Boo,
    y: Bar,
) { }

#[unsafe(no_mangle)]
pub extern "C" fn unsafe_root(
    x: Boo,
    y: Bar,
) { }
