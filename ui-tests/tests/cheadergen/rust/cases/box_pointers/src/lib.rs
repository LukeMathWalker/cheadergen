//! Box pointer types (`Box<T>`, `Option<Box<T>>`).
//!
//! Tests that cheadergen maps Rust box pointer types to the appropriate C pointer types.

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
