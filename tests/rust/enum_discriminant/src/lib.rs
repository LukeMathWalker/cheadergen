/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub const FOUR: i8 = 4;

#[repr(i8)]
enum E {
    A = 1,
    B = -1,
    C = 1 + 2,
    D = FOUR,
    F = (5),
    G = b'6' as i8,
    H = false as i8,
}

#[no_mangle]
pub extern "C" fn root(_: &E) {}
