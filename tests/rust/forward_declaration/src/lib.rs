/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[repr(C)]
struct TypeInfo {
    data: TypeData,
}

#[repr(C)]
enum TypeData {
    Primitive,
    Struct(StructInfo),
}

#[repr(C)]
struct StructInfo {
    fields: *const *const TypeInfo, // requires forward declaration
    num_fields: usize,
}

#[no_mangle]
pub extern "C" fn root(
    x: TypeInfo,
) {}
