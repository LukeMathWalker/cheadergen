/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// cbindgen:rename-all=CamelCase
#[no_mangle]
pub extern "C" fn test_camel_case(foo_bar: i32) {}

/// cbindgen:rename-all=PascalCase
#[no_mangle]
pub extern "C" fn test_pascal_case(foo_bar: i32) {}

/// cbindgen:rename-all=SnakeCase
#[no_mangle]
pub extern "C" fn test_snake_case(foo_bar: i32) {}

/// cbindgen:rename-all=ScreamingSnakeCase
#[no_mangle]
pub extern "C" fn test_screaming_snake_case(foo_bar: i32) {}

/// cbindgen:rename-all=GeckoCase
#[no_mangle]
pub extern "C" fn test_gecko_case(foo_bar: i32) {}

/// cbindgen:rename-all=prefix:prefix_
#[no_mangle]
pub extern "C" fn test_prefix(foo_bar: i32) {}
