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

// --- User-defined types ---

#[repr(C)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
pub enum Color {
    Red,
    Green,
    Blue,
}

#[repr(C)]
pub union Value {
    pub i: i32,
    pub f: f32,
}

#[repr(C)]
pub struct Container {
    pub point: Box<Point>,
}

// --- Box<T> with user-defined types ---

#[unsafe(no_mangle)]
pub extern "C" fn take_box_point(x: Box<Point>) {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn return_box_point() -> Box<Point> {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn take_box_color(x: Box<Color>) {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn take_box_value(x: Box<Value>) {
    todo!()
}

// --- Option<Box<T>> with user-defined types ---

#[unsafe(no_mangle)]
pub extern "C" fn take_option_box_point(x: Option<Box<Point>>) {
    todo!()
}

// --- Struct containing Box field ---

#[unsafe(no_mangle)]
pub extern "C" fn make_container() -> Container {
    todo!()
}
