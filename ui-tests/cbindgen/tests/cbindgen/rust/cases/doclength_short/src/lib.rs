/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// The root of all evil.
///
/// But at least it contains some more documentation as someone would expect
/// from a simple test case like this. Though, this shouldn't appear in the
/// output.
#[unsafe(no_mangle)]
pub extern "C" fn root() {
}

/// A little above the root, and a lot more visible, with a run-on sentence
/// to test going over the first line.
///
/// Still not here, though.
#[unsafe(no_mangle)]
pub extern "C" fn trunk() {
}
