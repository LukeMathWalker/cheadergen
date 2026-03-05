/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[no_mangle]
pub static mut MUT_GLOBAL_ARRAY: [c_char; 128] = [0; 128];

#[no_mangle]
pub static CONST_GLOBAL_ARRAY: [c_char; 128] = [0; 128];
