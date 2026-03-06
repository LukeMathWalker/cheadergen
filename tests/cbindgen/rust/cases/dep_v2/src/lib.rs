/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dep_2_dep::dep_struct;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_x(dep_struct: *const dep_struct) -> u32 {
    dep_struct.read().x
}
