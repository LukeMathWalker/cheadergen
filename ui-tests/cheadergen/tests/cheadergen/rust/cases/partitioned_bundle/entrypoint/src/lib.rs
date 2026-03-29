//! Target crate: uses dep by-value, with bundle = true in config.

use partitioned_bundle_dep::DepStruct;

#[unsafe(no_mangle)]
pub extern "C" fn use_dep(s: DepStruct) -> i32 {
    s.x
}
