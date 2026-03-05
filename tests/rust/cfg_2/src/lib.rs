/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[cfg(any(windows, unix))]
#[repr(C)]
struct Foo {
    x: i32,
}

#[cfg(windows)]
#[repr(C)]
struct Bar {
    y: Foo,
}

#[cfg(unix)]
#[repr(C)]
struct Bar {
    z: Foo,
}

#[repr(C)]
struct Root {
    w: Bar,
}

#[cfg(windows)]
pub const DEFAULT_X: i32 = 0x08;

#[cfg(unix)]
pub const DEFAULT_X: i32 = 0x2a;

#[no_mangle]
pub extern "C" fn root(a: Root)
{ }
