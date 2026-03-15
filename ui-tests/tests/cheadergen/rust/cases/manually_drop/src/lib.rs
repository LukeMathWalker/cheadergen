//! Test that `ManuallyDrop<T>` is simplified to `T` in all positions.

use std::mem::ManuallyDrop;

#[repr(C)]
pub struct Point {
    x: i32,
    y: i32,
}

#[repr(C)]
pub struct Wrapper {
    point: ManuallyDrop<Point>,
}

#[unsafe(no_mangle)]
pub extern "C" fn take_manually_drop(x: ManuallyDrop<Point>) {}

#[unsafe(no_mangle)]
pub extern "C" fn return_manually_drop() -> ManuallyDrop<Point> {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn ref_manually_drop(x: &ManuallyDrop<Point>) {}

#[unsafe(no_mangle)]
pub extern "C" fn take_wrapper(w: Wrapper) {}
