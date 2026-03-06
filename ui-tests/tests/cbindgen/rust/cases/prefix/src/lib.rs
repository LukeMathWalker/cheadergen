/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub const LEN: i32 = 22;

pub type NamedLenArray = [i32; LEN];
pub type ValuedLenArray = [i32; 22];

#[repr(u8)]
pub enum AbsoluteFontWeight {
    Weight(f32),
    Normal,
    Bold,
}

#[unsafe(no_mangle)]
pub extern "C" fn root(x: NamedLenArray, y: ValuedLenArray, z: AbsoluteFontWeight) {}

#[unsafe(no_mangle)]
pub const X: i64 = 22 << 22;

#[unsafe(no_mangle)]
pub const Y: i64 = X + X;
