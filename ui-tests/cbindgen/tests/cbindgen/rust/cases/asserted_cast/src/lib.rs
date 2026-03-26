/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// cbindgen:prefix-with-name
#[repr(C, u8)]
pub enum H {
    /// cbindgen:variant-mut-cast-attributes=MY_ATTRS
    Foo(i16),
    /// cbindgen:variant-const-cast-attributes=MY_ATTRS
    Bar { x: u8, y: i16 },
    /// cbindgen:variant-is-attributes=MY_ATTRS
    Baz
}

/// cbindgen:prefix-with-name
#[repr(C, u8, u16)]
pub enum I {
    /// cbindgen:variant-constructor-attributes=MY_ATTRS
    Foo(i16),
    /// cbindgen:eq-attributes=MY_ATTRS
    Bar { x: u8, y: i16 },
    Baz
}

/// cbindgen:prefix-with-name
#[repr(C, u8)]
pub enum J {
    Foo(i16),
    Bar { x: u8, y: i16 },
    Baz
}

/// cbindgen:prefix-with-name
#[repr(u8)]
pub enum K {
    Foo(i16),
    Bar { x: u8, y: i16 },
    Baz
}

#[unsafe(no_mangle)]
pub extern "C" fn foo(
    h: H,
    i: I,
    j: J,
    k: K,
) {}
