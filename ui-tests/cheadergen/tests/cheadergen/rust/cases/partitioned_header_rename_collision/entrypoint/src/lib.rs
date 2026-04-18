//! Target for the `partitioned_header_rename_collision` test.
//!
//! Two dependencies are renamed to the same `header_name`, which must fail
//! at config-validation time.

use partitioned_header_rename_collision_dep::DepA;
use partitioned_header_rename_collision_dep2::DepB;

#[unsafe(no_mangle)]
pub extern "C" fn use_both(a: DepA, b: DepB) -> i32 {
    a.x + b.y
}
