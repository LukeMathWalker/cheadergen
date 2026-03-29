//! Target crate: uses struct, union, and enum from dep all behind pointers.

use partitioned_forward_decl_kinds_dep::{MyEnum, MyStruct, MyUnion};

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
