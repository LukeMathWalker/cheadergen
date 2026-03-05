/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[repr(C)]
struct List<T> {
     members: *mut T,
     count: usize
}

struct A;

struct B;

#[no_mangle]
pub extern "C" fn foo(a: List<A>) { }

#[no_mangle]
pub extern "C" fn bar(b: List<B>) { }
