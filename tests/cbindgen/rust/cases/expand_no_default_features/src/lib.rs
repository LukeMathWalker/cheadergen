/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[repr(C)]
struct Foo {

}

#[cfg(feature = "extra_headers")]
#[unsafe(no_mangle)]
pub extern "C" fn extra_debug_fn() {
}

#[cfg(feature = "no_parse")]
pub extern "C" fn no_parse() {
    x;
}

#[cfg(feature = "cbindgen")]
#[unsafe(no_mangle)]
pub extern "C" fn cbindgen() {
}

#[unsafe(no_mangle)]
pub extern "C" fn root(a: Foo) {
}
