/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


pub const MY_CONST: u8 = 4;

#[unsafe(no_mangle)]
pub extern "C" fn ExternFunction() {
}

#[repr(u8)]
pub enum OnlyThisShouldBeGenerated {
    Foo,
    Bar,
}
