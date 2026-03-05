/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


#[repr(C)]
#[must_use]
pub struct OwnedPtr<T> {
    ptr: *mut T,
}

#[repr(C, u8)]
#[must_use]
pub enum MaybeOwnedPtr<T> {
    Owned(*mut T),
    None,
}

#[no_mangle]
#[must_use]
pub extern "C" fn maybe_consume(input: OwnedPtr<i32>) -> MaybeOwnedPtr<i32> {
}
