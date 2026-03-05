/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

mod uhoh {
    enum BindingType { Buffer, NotBuffer }
}

#[repr(u32)]
pub enum BindingType { Buffer = 0, NotBuffer = 1 }

#[repr(C)]
pub struct BindGroupLayoutEntry {
    pub ty: BindingType, // This is the repr(u32) enum
}

#[no_mangle]
pub extern "C" fn root(entry: BindGroupLayoutEntry) {}
