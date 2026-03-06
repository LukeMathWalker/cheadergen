/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


struct Opaque();

#[repr(C)]
struct Foo {
    x: Option<&Opaque>,
    y: Option<&mut Opaque>,
    z: Option<fn () -> ()>,
    zz: *mut Option<fn () -> ()>,
}

#[repr(C)]
union Bar {
    x: Option<&Opaque>,
    y: Option<&mut Opaque>,
    z: Option<fn () -> ()>,
    zz: *mut Option<fn () -> ()>,
}

#[unsafe(no_mangle)]
pub extern "C" fn root(
	a: Option<&Opaque>,
    b: Option<&mut Opaque>,
    c: Foo,
    d: Bar,
    e: *mut Option<*mut Opaque>,
    f: extern "C" fn(Option<&Opaque>),
) { }
