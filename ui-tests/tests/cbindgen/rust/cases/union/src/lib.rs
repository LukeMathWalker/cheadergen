/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::marker::PhantomData;

union Opaque {
    x: i32,
    y: f32,
}

#[repr(C)]
union Normal {
    x: i32,
    y: f32,
}

#[repr(C)]
union NormalWithZST {
    x: i32,
    y: f32,
    z: (),
    w: PhantomData<i32>,
}

#[unsafe(no_mangle)]
pub extern "C" fn root(
    a: *mut Opaque,
    b: Normal,
    c: NormalWithZST
) { }
