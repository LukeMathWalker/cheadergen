/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[repr(C)]
struct Foo {}

impl Foo {
    #[cheadergen::config(export)]
    pub const GA: i32 = 10;
    #[cheadergen::config(export)]
    pub const BU: &'static str = "hello world";
    #[cheadergen::config(export)]
    pub const ZO: f32 = 3.14;

    pub(crate) const DONT_EXPORT_CRATE: i32 = 20;
    const DONT_EXPORT_PRIV: i32 = 30;
}

#[unsafe(no_mangle)]
#[expect(improper_ctypes_definitions)]
pub extern "C" fn root(x: Foo) { }
