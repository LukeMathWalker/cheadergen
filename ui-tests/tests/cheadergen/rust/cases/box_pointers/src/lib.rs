//! Box pointer types (`Box<T>`, `Option<Box<T>>`).
//!
//! `Box<T>` is among the types that benefit from
//! [null pointer optimization](https://doc.rust-lang.org/std/option/#representation):
//! `Option<Box<T>>` has the same size and ABI as a raw pointer,
//! using null to represent `None`.
//!
//! cbindgen maps `Box<T>` to `T *` and `Option<Box<T>>` to `T *` (nullable).

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
