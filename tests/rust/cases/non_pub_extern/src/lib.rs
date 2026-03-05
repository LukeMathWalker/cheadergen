/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[no_mangle]
static FIRST: u32 = 10;

#[export_name = "RENAMED"]
static SECOND: u32 = 42;

#[no_mangle]
extern "C" fn first()
{ }

#[export_name = "renamed"]
extern fn second()
{ }
