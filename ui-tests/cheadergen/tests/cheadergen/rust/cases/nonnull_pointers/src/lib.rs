//! NonNull pointer types (`NonNull<T>`, `Option<NonNull<T>>`).
//!
//! `NonNull<T>` is among the types that benefit from
//! [null pointer optimization](https://doc.rust-lang.org/std/option/#representation):
//! `Option<NonNull<T>>` has the same size and ABI as a raw pointer,
//! using null to represent `None`.
//!
//! cbindgen maps `NonNull<T>` to `T *` and `Option<NonNull<T>>` to `T *` (nullable).

use std::ptr::NonNull;

// --- NonNull<T> ---

#[unsafe(no_mangle)]
pub extern "C" fn take_nonnull_i32(x: NonNull<i32>) {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn return_nonnull_i32() -> NonNull<i32> {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn take_nonnull_f32(x: NonNull<f32>) {
    todo!()
}

// --- Option<NonNull<T>> ---

#[unsafe(no_mangle)]
pub extern "C" fn take_option_nonnull_i32(x: Option<NonNull<i32>>) {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn return_option_nonnull_i32() -> Option<NonNull<i32>> {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn take_option_nonnull_u8(x: Option<NonNull<u8>>) {
    todo!()
}
