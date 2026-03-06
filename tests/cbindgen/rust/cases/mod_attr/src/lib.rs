/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[cfg(foo)]
pub const FOO: i32 = 1;

#[cfg(foo)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn foo(foo: &Foo) {}

#[cfg(foo)]
#[repr(C)]
pub struct Foo {}

#[cfg(feature = "foobar")]
pub mod foo {
    #[cfg(bar)]
    pub const BAR: i32 = 2;

    #[cfg(bar)]
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn bar(bar: &Bar) {}

    #[cfg(bar)]
    #[repr(C)]
    pub struct Bar {}
}
