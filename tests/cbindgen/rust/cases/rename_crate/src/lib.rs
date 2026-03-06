/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unused_variables)]

extern crate dependency as internal_name;
extern crate renamed_dep;

pub use internal_name::*;
pub use renamed_dep::*;

#[unsafe(no_mangle)]
pub extern "C" fn root(a: Foo) {
}

#[unsafe(no_mangle)]
pub extern "C" fn renamed_func(a: RenamedTy) {
}


#[unsafe(no_mangle)]
pub extern "C" fn no_extern_func(a: ContainsNoExternTy) {
}
