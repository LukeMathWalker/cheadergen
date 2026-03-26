/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[repr(C)]
pub struct MyStruct<'a> {
    number: std::mem::MaybeUninit<&'a i32>,
}

pub struct NotReprC<T> {
    inner: T,
}

pub type Foo<'a> = NotReprC<std::mem::MaybeUninit<&'a i32>>;

#[unsafe(no_mangle)]
pub extern "C" fn root<'a, 'b>(a: &'a Foo, with_maybe_uninit: &'b MyStruct) {}
