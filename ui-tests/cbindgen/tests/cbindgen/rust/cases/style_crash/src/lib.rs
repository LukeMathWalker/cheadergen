/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub trait SpecifiedValueInfo {
    const SUPPORTED_TYPES: u8 = 0;
}

impl<T: SpecifiedValueInfo> SpecifiedValueInfo for [T] {
    const SUPPORTED_TYPES: u8 = T::SUPPORTED_TYPES;
}
