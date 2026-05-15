/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[cheadergen::config(export)]
pub const SIZE: isize = 4;

#[repr(C)]
pub struct WithoutAs {
    items: [char; SIZE as usize],
}

#[repr(C)]
pub struct WithAs {
    items: [char; SIZE as usize],
}

// dummy function to make `WithoutAs` and `WithAs` part of the public api
#[unsafe(no_mangle)]
#[expect(improper_ctypes_definitions)]
pub extern "C" fn some_fn(a: WithoutAs, b: WithAs) {

}
