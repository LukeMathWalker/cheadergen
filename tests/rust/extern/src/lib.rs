/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[repr(C)]
struct Normal {
    x: i32,
    y: f32,
}

extern "C" {
    fn foo() -> i32;

    fn bar(a: Normal);
}

extern {
    fn baz() -> i32;
}
