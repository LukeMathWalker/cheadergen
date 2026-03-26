/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(c_variadic)]

use std::ffi::VaList;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn va_list_test(count: i32, mut ap: VaList) -> i32 {
    ap.arg()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn va_list_test2(count: i32, mut ap: ...) -> i32 {
    ap.arg()
}

type VaListFnPtr = Option<unsafe extern "C" fn(count: i32, VaList) -> i32>;
type VaListFnPtr2 = Option<unsafe extern "C" fn(count: i32, ...) -> i32>;

#[repr(C)]
struct Interface<T> {
    fn1: T,
}

#[unsafe(no_mangle)]
pub extern "C" fn va_list_fn_ptrs(
    fn1: Option<unsafe extern "C" fn(count: i32, VaList) -> i32>,
    fn2: Option<unsafe extern "C" fn(count: i32, ...) -> i32>,
    fn3: VaListFnPtr,
    fn4: VaListFnPtr2,
    fn5: Interface<Option<unsafe extern "C" fn(count: i32, VaList) -> i32>>,
    fn6: Interface<Option<unsafe extern "C" fn(count: i32, ...) -> i32>>,
) {
}
