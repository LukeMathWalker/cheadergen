//! Target crate for the `partitioned_basic` test.

use partitioned_basic_dep::DepStruct;

#[unsafe(no_mangle)]
pub extern "C" fn use_dep(s: DepStruct) -> i32 {
    s.x + s.y
}
