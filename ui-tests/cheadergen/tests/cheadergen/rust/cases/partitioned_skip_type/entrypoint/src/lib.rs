//! Target crate: uses `Visible` by-value and `Hidden` behind a pointer.

use partitioned_skip_type_dep::{Hidden, Visible};

#[unsafe(no_mangle)]
pub extern "C" fn use_visible(v: Visible) -> i32 {
    v.value
}

#[unsafe(no_mangle)]
pub extern "C" fn use_hidden(ptr: *const Hidden) -> *const Hidden {
    ptr
}
