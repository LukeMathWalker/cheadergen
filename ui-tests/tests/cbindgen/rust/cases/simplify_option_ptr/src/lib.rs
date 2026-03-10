/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::mem::ManuallyDrop;

struct Opaque();

#[repr(C)]
struct Foo<'a> {
    x: Option<&'a Opaque>,
    y: Option<&'a mut Opaque>,
    z: Option<fn () -> ()>,
    zz: *mut Option<fn () -> ()>,
}

#[repr(C)]
union Bar<'a> {
    x: Option<&'a Opaque>,
    y: ManuallyDrop<Option<&'a mut Opaque>>,
    z: Option<fn () -> ()>,
    zz: *mut Option<fn () -> ()>,
}

#[unsafe(no_mangle)]
pub extern "C" fn root<'a>(
	a: Option<&'a Opaque>,
    b: Option<&'a mut Opaque>,
    c: Foo<'a>,
    d: Bar<'a>,
    e: *mut Option<*mut Opaque>,
    f: extern "C" fn(Option<&Opaque>),
) { }
