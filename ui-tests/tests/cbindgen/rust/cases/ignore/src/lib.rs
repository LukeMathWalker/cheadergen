/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// cbindgen:ignore
#[unsafe(no_mangle)]
pub extern "C" fn root() {}

/// cbindgen:ignore
///
/// Something else.
#[unsafe(no_mangle)]
pub extern "C" fn another_root() {}

#[unsafe(no_mangle)]
pub extern "C" fn no_ignore_root() {}

/// cbindgen:ignore
#[repr(C)]
pub struct IgnoreStruct {}

pub struct IgnoreStructWithImpl;

/// cbindgen:ignore
impl IgnoreStructWithImpl {
    #[unsafe(no_mangle)]
    pub extern "C" fn ignore_associated_method() {}

    pub const IGNORE_INNER_CONST: u32 = 0;
}

/// cbindgen:ignore
pub const IGNORE_CONST: u32 = 0;

pub const NO_IGNORE_CONST: u32 = 0;

pub struct NoIgnoreStructWithImpl;

impl NoIgnoreStructWithImpl {
    /// cbindgen:ignore
    #[unsafe(no_mangle)]
    pub extern "C" fn ignore_associated_method() {}

    #[unsafe(no_mangle)]
    pub extern "C" fn no_ignore_associated_method() {}

    /// cbindgen:ignore
    pub const IGNORE_INNER_CONST: u32 = 0;

    pub const NO_IGNORE_INNER_CONST: u32 = 0;
}

/// cbindgen:ignore
enum IgnoreEnum {}

enum NoIgnoreEnum {}
