//! Target crate: uses DepType1 by-value and DepType2 behind a pointer.

use partitioned_mixed_usage_dep::{DepType1, DepType2};

#[unsafe(no_mangle)]
pub extern "C" fn use_by_value(s: DepType1) -> i32 {
    s.a
}

#[unsafe(no_mangle)]
pub extern "C" fn use_by_pointer(ptr: *const DepType2) -> *const DepType2 {
    ptr
}
