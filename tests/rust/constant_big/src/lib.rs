/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub const UNSIGNED_NEEDS_ULL_SUFFIX: u64       = 0x8000_0000_0000_0000;
pub const UNSIGNED_DOESNT_NEED_ULL_SUFFIX: u64 = 0x7000_0000_0000_0000;

// i64::min_value()
pub const SIGNED_NEEDS_ULL_SUFFIX: i64         = -9223372036854775808;

// i64::min_value() + 1
pub const SIGNED_DOESNT_NEED_ULL_SUFFIX: i64   = -9223372036854775807;
