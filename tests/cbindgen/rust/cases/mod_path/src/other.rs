/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[repr(C)]
pub struct ExportMe {
    val: u64
}

#[repr(C)]
pub struct DoNotExportMe {
    val: u64
}

pub const EXPORT_ME_TOO: u8 = 0x2a;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn export_me(val: *mut ExportMe) { }
