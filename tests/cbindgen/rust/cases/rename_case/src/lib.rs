/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// cbindgen:rename-all=CamelCase
#[unsafe(no_mangle)]
pub extern "C" fn test_camel_case(foo_bar: i32) {}

/// cbindgen:rename-all=PascalCase
#[unsafe(no_mangle)]
pub extern "C" fn test_pascal_case(foo_bar: i32) {}

/// cbindgen:rename-all=SnakeCase
#[unsafe(no_mangle)]
pub extern "C" fn test_snake_case(foo_bar: i32) {}

/// cbindgen:rename-all=ScreamingSnakeCase
#[unsafe(no_mangle)]
pub extern "C" fn test_screaming_snake_case(foo_bar: i32) {}

/// cbindgen:rename-all=GeckoCase
#[unsafe(no_mangle)]
pub extern "C" fn test_gecko_case(foo_bar: i32) {}

/// cbindgen:rename-all=prefix:prefix_
#[unsafe(no_mangle)]
pub extern "C" fn test_prefix(foo_bar: i32) {}
