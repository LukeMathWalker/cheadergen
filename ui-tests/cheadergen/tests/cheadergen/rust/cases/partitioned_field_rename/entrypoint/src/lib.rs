//! Target crate: uses tuple struct with renamed fields from dep.

use partitioned_field_rename_dep::Vec2;

#[unsafe(no_mangle)]
pub extern "C" fn use_vec2(v: Vec2) -> f32 {
    v.0 + v.1
}
