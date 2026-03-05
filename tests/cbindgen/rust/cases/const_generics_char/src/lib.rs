/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::marker::PhantomData;

#[repr(C)]
struct TakeUntil<'a, const V: char>
{
    marker: PhantomData<&'a str>,
    start: *const u8,
    len: usize,
    point: usize,
}

#[no_mangle]
pub unsafe extern "C" fn until_nul(start: *const u8, len: usize) -> TakeUntil<'a, '\0'> {
    TakeUntil {
        marker: PhantomData,
        start,
        len,
        point: 0,
    }
}
