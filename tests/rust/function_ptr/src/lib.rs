/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub type MyCallback = Option<unsafe extern "C" fn(a: usize, b: usize)>;

pub type MyOtherCallback = Option<unsafe extern "C" fn(a: usize, lot: usize, of: usize, args: usize, and_then_some: usize)>;

#[no_mangle]
pub extern "C" fn my_function(a: MyCallback, b: MyOtherCallback) {}
