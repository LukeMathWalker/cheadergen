//! Target crate: uses both BStruct (which contains Wrapper<i32>) and
//! Wrapper<i32> directly. Both headers should have Wrapper_i32 with guards.

use partitioned_generic_overlap_dep::BStruct;
use partitioned_generic_overlap_dep2::Wrapper;

#[unsafe(no_mangle)]
pub extern "C" fn use_b(s: BStruct) -> i32 {
    s.wrapped.value
}

#[unsafe(no_mangle)]
pub extern "C" fn use_wrapper(w: Wrapper<i32>) -> i32 {
    w.value
}
