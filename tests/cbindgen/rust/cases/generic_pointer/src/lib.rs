/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[repr(C)]
pub struct Foo<T> {
    a: T,
}

pub type Boo = Foo<*mut u8>;

#[unsafe(no_mangle)]
pub extern "C" fn root(
    x: Boo,
) { }

#[unsafe(no_mangle)]
pub extern "C" fn my_function(x: Foo<[u8; 4]>) {}
