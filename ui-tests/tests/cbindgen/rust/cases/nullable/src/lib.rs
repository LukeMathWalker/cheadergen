/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::ptr::NonNull;

struct Opaque;

#[repr(C)]
pub struct Foo<T> {
    a: NonNull<f32>,
    b: NonNull<T>,
    c: NonNull<Opaque>,
    d: NonNull<NonNull<T>>,
    e: NonNull<NonNull<f32>>,
    f: NonNull<NonNull<Opaque>>,
    g: Option<NonNull<T>>,
    h: Option<NonNull<i32>>,
    i: Option<NonNull<NonNull<i32>>>,
}

#[unsafe(no_mangle)]
pub extern "C" fn root(arg: NonNull<i32>, foo: *mut Foo<u64>, d: NonNull<NonNull<Opaque>>) { }
