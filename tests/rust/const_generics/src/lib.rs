/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[repr(transparent)]
pub struct CArrayString<const CAP: usize> {
    pub chars: [i8; CAP],
}

pub const TITLE_SIZE: usize = 80;

#[repr(C)]
pub struct Book {
    pub title: CArrayString<TITLE_SIZE>,
    pub author: CArrayString<40>,
}

#[no_mangle]
pub extern "C" fn root(a: *mut Book) {}
