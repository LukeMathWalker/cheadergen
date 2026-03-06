/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[repr(C)]
pub struct FixedPoint<const FRACTION_BITS: u16> {
    value: u16,
}

pub const FONT_WEIGHT_FRACTION_BITS: u16 = 6;

pub type FontWeightFixedPoint = FixedPoint<FONT_WEIGHT_FRACTION_BITS>;

#[repr(C)]
pub struct FontWeight(FontWeightFixedPoint);

impl FontWeight {
    pub const NORMAL: FontWeight = FontWeight(FontWeightFixedPoint { value: 400 << FONT_WEIGHT_FRACTION_BITS });
}

#[unsafe(no_mangle)]
pub extern "C" fn root(w: FontWeight) {}
