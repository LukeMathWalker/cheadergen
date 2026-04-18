//! Target for the `partitioned_header_rename_invalid` test.
//!
//! `header_name` contains a path separator, which must be rejected at
//! config-validation time.

use partitioned_header_rename_invalid_dep::DepStruct;

#[unsafe(no_mangle)]
pub extern "C" fn use_dep(s: DepStruct) -> i32 {
    s.x
}
