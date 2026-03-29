//! Target crate: uses BStruct which contains Wrapper<i32> from dep2.

use partitioned_generic_in_dep_struct_dep::BStruct;

#[unsafe(no_mangle)]
pub extern "C" fn use_b(s: BStruct) -> i32 {
    s.wrapped.value
}
