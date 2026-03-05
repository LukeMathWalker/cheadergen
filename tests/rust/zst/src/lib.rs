/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[repr(C)]
pub struct TraitObject {
    pub data: *mut (),
    pub vtable: *mut (),
}

#[no_mangle]
pub extern "C" fn root(ptr: *const (), t: TraitObject) -> *mut () {
    std::ptr::null_mut()
}
