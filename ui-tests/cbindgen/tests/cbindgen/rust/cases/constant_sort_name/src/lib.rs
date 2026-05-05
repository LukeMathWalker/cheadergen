/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[cheadergen::config(export)]
pub const B: u8 = 0;
#[cheadergen::config(export)]
pub const A: u8 = 0;

#[unsafe(no_mangle)]
pub static D: u8 = 0;
#[unsafe(no_mangle)]
pub static C: u8 = 0;
