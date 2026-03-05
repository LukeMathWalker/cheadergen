/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[no_mangle]
pub extern "C" fn loop_forever() -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn normal_return(arg: Example, other: extern "C" fn(u8) -> !) -> u8 {
    0
}

#[repr(C)]
pub struct Example {
    pub f: extern "C" fn(usize, usize) -> !,
}
