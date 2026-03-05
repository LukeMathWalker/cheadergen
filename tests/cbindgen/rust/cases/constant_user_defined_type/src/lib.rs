/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[repr(C)]
pub struct S {
    field: u8,
}

/// cbindgen:enum-class=false
#[repr(C)]
pub enum E {
    V,
}
use E::*;

pub type A = u8;

pub const C1: S = S { field: 0 };
pub const C2: E = V;
pub const C3: A = 0;
