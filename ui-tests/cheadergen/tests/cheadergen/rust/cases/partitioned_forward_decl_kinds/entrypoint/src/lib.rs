//! Target crate: uses struct, union, enum, type alias, and transparent
//! wrapper from dep all behind pointers.

use partitioned_forward_decl_kinds_dep::{MyAlias, MyEnum, MyStruct, MyTransparent, MyUnion};

#[unsafe(no_mangle)]
pub extern "C" fn use_struct_ptr(ptr: *const MyStruct) -> *const MyStruct {
    ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn use_union_ptr(ptr: *const MyUnion) -> *const MyUnion {
    ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn use_enum_ptr(ptr: *const MyEnum) -> *const MyEnum {
    ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn use_alias_ptr(ptr: *const MyAlias) -> *const MyAlias {
    ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn use_transparent_ptr(ptr: *const MyTransparent) -> *const MyTransparent {
    ptr
}
