/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[doc = "With doc attr, each attr contribute to one line of document"]
#[doc = "like this one with a new line character at its end"]
#[doc = "and this one as well. So they are in the same paragraph"]
#[doc = ""]
#[doc = "We treat empty doc comments as empty lines, so they break to the next paragraph."]
#[doc = ""]
#[doc = "Newlines are preserved with leading spaces added\nto prettify and avoid misinterpreting leading symbols."]
#[doc = "like headings and lists."]
#[doc = ""]
#[doc = "Line ends with two new lines\n\nShould break to next paragraph"]
#[no_mangle]
pub extern "C" fn root() {}
