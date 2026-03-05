/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[repr(C)]
struct Foo {}

impl Foo {
    const FOO: i32 = 0;
}

pub const Foo_FOO: u32 = 42;

