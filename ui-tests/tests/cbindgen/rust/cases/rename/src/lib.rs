/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

struct A {
    x: i32,
    y: f32,
}

#[repr(C)]
struct B {
    x: i32,
    y: f32,
}

union C {
    x: i32,
    y: f32,
}

#[repr(C)]
union D {
    x: i32,
    y: f32,
}

#[repr(u8)]
enum E {
    x = 0,
    y = 1,
}

type F = A;

#[unsafe(no_mangle)]
pub static G: i32 = 10;

pub const H: i32 = 10;

pub const I: isize = 10 as *mut F as isize;

#[unsafe(no_mangle)]
pub extern "C" fn root(
    a: *const A,
    b: B,
    c: C,
    d: D,
    e: E,
    f: F,
) { }

