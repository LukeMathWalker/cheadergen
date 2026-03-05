/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[repr(C)]
pub struct Point {
    x: i32,
    y: i32,
}

#[repr(C)]
pub struct MyStruct {
    point: std::mem::ManuallyDrop<Point>,
}

pub struct NotReprC<T> {
    inner: T,
}

pub type Foo = NotReprC<std::mem::ManuallyDrop<Point>>;

#[no_mangle]
pub extern "C" fn root(a: &Foo, with_manual_drop: &MyStruct) {}

#[no_mangle]
pub extern "C" fn take(with_manual_drop: std::mem::ManuallyDrop<Point>) {}
