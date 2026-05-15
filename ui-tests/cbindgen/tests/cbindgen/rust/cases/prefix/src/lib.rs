/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[cheadergen::config(export)]
pub const LEN: usize = 22;

pub type NamedLenArray = [i32; LEN];
pub type ValuedLenArray = [i32; 22];

#[repr(u8)]
pub enum AbsoluteFontWeight {
    Weight(f32),
    Normal,
    Bold,
}

#[unsafe(no_mangle)]
#[expect(improper_ctypes_definitions)]
pub extern "C" fn root(x: NamedLenArray, y: ValuedLenArray, z: AbsoluteFontWeight) {}

#[expect(no_mangle_const_items)]
#[unsafe(no_mangle)]
#[cheadergen::config(export)]
pub const X: i64 = 22 << 22;

#[expect(no_mangle_const_items)]
#[unsafe(no_mangle)]
#[cheadergen::config(export)]
pub const Y: i64 = X + X;
