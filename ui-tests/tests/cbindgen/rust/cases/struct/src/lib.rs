/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::marker::PhantomData;

struct Opaque {
    x: i32,
    y: f32,
}

#[repr(C)]
struct Normal {
    x: i32,
    y: f32,
}

#[repr(C)]
struct NormalWithZST {
    x: i32,
    y: f32,
    z: (),
    w: PhantomData<i32>,
    v: PhantomPinned,
}

/// cbindgen:rename-all=GeckoCase
#[repr(C)]
struct TupleRenamed(i32, f32);

/// cbindgen:field-names=[x, y]
#[repr(C)]
struct TupleNamed(i32, f32);

#[unsafe(no_mangle)]
pub extern "C" fn root(
    a: *mut Opaque,
    b: Normal,
    c: NormalWithZST,
    d: TupleRenamed,
    e: TupleNamed
) { }
