//! Optional references (`Option<&T>`, `Option<&mut T>`).
//!
//! `&T` and `&mut T` are among the types that benefit from
//! [null pointer optimization](https://doc.rust-lang.org/std/option/#representation):
//! `Option<&T>` and `Option<&mut T>` have the same size and ABI as a raw pointer,
//! using null to represent `None`.
//!
//! cbindgen maps these to nullable C pointers (`const T *` / `T *`).

/// A simple `#[repr(C)]` struct to test optional references to user-defined types.
#[repr(C)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

// --- Option<&T> ---

#[unsafe(no_mangle)]
pub extern "C" fn take_option_ref_i32(x: Option<&i32>) {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn return_option_ref_i32() -> Option<&'static i32> {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn take_option_ref_f32(x: Option<&f32>) {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn take_option_ref_point(x: Option<&Point>) {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn return_option_ref_point() -> Option<&'static Point> {
    todo!()
}

// --- Option<&mut T> ---

#[unsafe(no_mangle)]
pub extern "C" fn take_option_mut_ref_i32(x: Option<&mut i32>) {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn return_option_mut_ref_i32() -> Option<&'static mut i32> {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn take_option_mut_ref_point(x: Option<&mut Point>) {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn return_option_mut_ref_point() -> Option<&'static mut Point> {
    todo!()
}
