//! Target crate for the `partitioned_header_rename` test.
//!
//! Exercises `[package.<name>] header_name = "..."`: the dep's header is
//! written to `custom_dep.h` and the entrypoint `#include`s that path.

use partitioned_header_rename_dep::DepStruct;

#[unsafe(no_mangle)]
pub extern "C" fn use_dep(s: DepStruct) -> i32 {
    s.x + s.y
}
