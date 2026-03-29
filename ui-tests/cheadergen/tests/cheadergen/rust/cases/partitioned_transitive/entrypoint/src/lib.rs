//! Target crate: uses BStruct from dep, which itself contains CStruct from dep2.

use partitioned_transitive_dep::BStruct;

#[unsafe(no_mangle)]
pub extern "C" fn use_b(s: BStruct) -> i32 {
    s.c.value
}
