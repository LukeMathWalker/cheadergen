/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[repr(C)]
pub struct ArrayVec<T, const CAP: usize> {
    // the `len` first elements of the array are initialized
    xs: [T; CAP],
    len: u32,
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn push(v: *mut ArrayVec<*mut u8, 100>, elem: *mut u8) -> i32 {
    if (*v).len < 100 {
        (*v).xs[(*v).len] = elem;
        (*v).len += 1;
        1
    } else {
        0
    }
}
