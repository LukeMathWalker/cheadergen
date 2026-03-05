/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[repr(C)]
struct A {
    namespace: i32,
    float: f32,
}

/// cbindgen:field-names=[namespace, float]
#[repr(C)]
struct B(i32, f32);

#[repr(C, u8)]
enum C {
    D { namespace: i32, float: f32 },
}

#[repr(C, u8)]
enum E {
    Double(f64),
    Float(f32),
}

#[repr(C, u8)]
enum F {
    double(f64),
    float(f32),
}

#[no_mangle]
pub extern "C" fn root(
    a: A,
    b: B,
    c: C,
    e: E,
    f: F,
    namespace: i32,
    float: f32,
) { }
