/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// Name mangling can cope with char-like byte literals.

#[repr(C)]
pub struct Parser<const OPEN: u8, const CLOSE: u8> {
    pub buf: *mut u8,
    pub len: usize,
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn init_parens_parser(p: *mut Parser<b'(', b')'>, buf: *mut u8, len: usize) {
    unsafe {
        *p = Parser { buf, len };
    }
}

// The same type as above, because `b'(' == 40 && b')' == 41`. And it happens
// to mangle to the same C identifier. It doesn't always work out that way!
#[unsafe(no_mangle)]
pub unsafe extern "C" fn destroy_parens_parser(p: *mut Parser<40, 41>) {
    // nothing to do
}


#[unsafe(no_mangle)]
pub unsafe extern "C" fn init_braces_parser(p: *mut Parser<b'{', b'}'>, buf: *mut u8, len: usize) {
    unsafe {
        *p = Parser { buf, len };
    }
}
