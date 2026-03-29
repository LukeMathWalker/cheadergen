//! Target crate: uses generic instantiations from dep.

use partitioned_generic_dep::Wrapper;

#[unsafe(no_mangle)]
pub extern "C" fn use_wrapper_i32(w: Wrapper<i32>) -> Wrapper<i32> {
    w
}

#[unsafe(no_mangle)]
pub extern "C" fn use_wrapper_f32(w: Wrapper<f32>) -> Wrapper<f32> {
    w
}
