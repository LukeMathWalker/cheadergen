//! Pointer-wrapper types (`NonNull<T>`, `Option<NonNull<T>>`, `Box<T>`, `Option<Box<T>>`).
//!
//! Tests that cheadergen maps Rust pointer-wrapper types to the appropriate C pointer types.

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

// --- Box<T> ---

#[unsafe(no_mangle)]
pub extern "C" fn take_box_i32(x: Box<i32>) {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn return_box_i32() -> Box<i32> {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn take_box_f32(x: Box<f32>) {
    todo!()
}

// --- Option<Box<T>> ---

#[unsafe(no_mangle)]
pub extern "C" fn take_option_box_i32(x: Option<Box<i32>>) {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn return_option_box_i32() -> Option<Box<i32>> {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn take_option_box_u8(x: Option<Box<u8>>) {
    todo!()
}
