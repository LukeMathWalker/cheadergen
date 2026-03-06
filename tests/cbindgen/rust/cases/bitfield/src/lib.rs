/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[repr(C)]
pub struct HasBitfields {
    #[cfg(not(feature = "cbindgen"))]
    foo_and_bar: u64,

    #[cfg(feature = "cbindgen")]
    /// cbindgen:bitfield=8
    foo: u64,
    #[cfg(feature = "cbindgen")]
    /// cbindgen:bitfield=56
    bar: u64,
}

#[unsafe(no_mangle)]
pub extern "C" fn root(_: &HasBitfields) {}
