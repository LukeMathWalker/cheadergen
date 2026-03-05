/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[export_name = "do_the_thing_with_export_name"]
pub extern "C" fn do_the_thing() {
  println!("doing the thing!");
}

#[unsafe(export_name = "do_the_thing_with_unsafe_export_name")]
pub extern "C" fn unsafe_do_the_thing() {
  println!("doing the thing!");
}
