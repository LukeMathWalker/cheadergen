/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[repr(C)]
pub struct RenamedTy {
    y: u64,
}

#[cfg(not(target_os = "freebsd"))]
#[repr(C)]
pub struct ContainsNoExternTy {
    pub field: no_extern::NoExternTy,
}

#[cfg(target_os = "freebsd")]
#[repr(C)]
pub struct ContainsNoExternTy {
    pub field: u64,
}
