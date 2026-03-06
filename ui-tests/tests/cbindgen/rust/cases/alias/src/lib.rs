/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[repr(C)]
struct Dep {
    a: i32,
    b: f32,
}

#[repr(C)]
struct Foo<X> {
    a: X,
    b: X,
    c: Dep,
}

#[repr(u32)]
enum Status {
    Ok,
    Err,
}

type IntFoo = Foo<i32>;
type DoubleFoo = Foo<f64>;

type Unit = i32;
type SpecialStatus = Status;

#[unsafe(no_mangle)]
pub extern "C" fn root(
    x: IntFoo,
    y: DoubleFoo,
    z: Unit,
    w: SpecialStatus
) { }
