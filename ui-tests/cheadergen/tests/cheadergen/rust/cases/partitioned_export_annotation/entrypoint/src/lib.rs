//! Target crate: has functions but does NOT reference the dep's exported type.

#[unsafe(no_mangle)]
pub extern "C" fn standalone() -> i32 {
    42
}
