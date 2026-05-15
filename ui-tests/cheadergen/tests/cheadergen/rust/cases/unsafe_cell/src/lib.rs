//! Test that `UnsafeCell<T>` is simplified to `T` in all positions,
//! and that `UnsafeCell` correctly disables null pointer optimization.

use std::cell::UnsafeCell;
use std::mem::ManuallyDrop;
use std::ptr::NonNull;

#[repr(C)]
pub struct Point {
    x: i32,
    y: i32,
}

#[repr(C)]
pub struct Wrapper {
    point: UnsafeCell<Point>,
}

/// `UnsafeCell<T>` as parameter → `T`
#[unsafe(no_mangle)]
pub extern "C" fn take_unsafe_cell(x: UnsafeCell<Point>) {}

/// `UnsafeCell<T>` as return type → `T`
#[unsafe(no_mangle)]
pub extern "C" fn return_unsafe_cell() -> UnsafeCell<Point> {
    todo!()
}

/// `&UnsafeCell<T>` as parameter → `const T *`
#[unsafe(no_mangle)]
pub extern "C" fn ref_unsafe_cell(x: &UnsafeCell<Point>) {}

/// `ManuallyDrop<UnsafeCell<T>>` → `T` (both transforms compose)
#[unsafe(no_mangle)]
pub extern "C" fn nested_manually_drop_unsafe_cell(x: ManuallyDrop<UnsafeCell<Point>>) {}

/// `Option<UnsafeCell<NonNull<Point>>>` must NOT be simplified via NPO.
/// `UnsafeCell` disables niche optimization even though `NonNull` is NPO-eligible.
#[unsafe(no_mangle)]
#[expect(improper_ctypes_definitions)]
pub extern "C" fn option_unsafe_cell_nonnull(
    x: Option<UnsafeCell<NonNull<Point>>>,
) {}

/// `Option<NonNull<UnsafeCell<Point>>>` SHOULD simplify to `*mut T`.
/// NonNull provides its own niche (non-null pointer), so UnsafeCell nested inside
/// does not prevent NPO.
#[unsafe(no_mangle)]
pub extern "C" fn option_nonnull_unsafe_cell(
    x: Option<NonNull<UnsafeCell<Point>>>,
) {}
