/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[repr(C)]
union Foo<T> {
    data: *const T
}

union Bar<T> {
    data: *const T
}

#[repr(C)]
union Tuple<T, E> {
    a: *const T,
    b: *const E,
}

type Indirection<T> = Tuple<T, f32>;

#[unsafe(no_mangle)]
pub extern "C" fn root(
    a: Foo<i32>,
    b: Foo<f32>,
    c: Bar<f32>,
    d: Foo<Bar<f32>>,
    e: Bar<Foo<f32>>,
    f: Bar<Bar<f32>>,
    g: Tuple<Foo<f32>, f32>,
    h: Indirection<f32>
) { }
