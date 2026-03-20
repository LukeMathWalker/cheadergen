//! Test that `MaybeUninit<T>` is simplified to `T` in all positions,
//! and that `MaybeUninit` correctly disables null pointer optimization.

use std::mem::MaybeUninit;
use std::mem::ManuallyDrop;
use std::ptr::NonNull;

#[repr(C)]
pub struct Point {
    x: i32,
    y: i32,
}

#[repr(C)]
pub struct Wrapper {
    point: MaybeUninit<Point>,
}

/// `MaybeUninit<T>` in a struct field → `T`
#[unsafe(no_mangle)]
pub extern "C" fn take_wrapper(x: Wrapper) {}

/// `MaybeUninit<T>` as parameter → `T`
#[unsafe(no_mangle)]
pub extern "C" fn take_maybe_uninit(x: MaybeUninit<Point>) {}

/// `MaybeUninit<T>` as return type → `T`
#[unsafe(no_mangle)]
pub extern "C" fn return_maybe_uninit() -> MaybeUninit<Point> {
    todo!()
}

/// `&MaybeUninit<T>` as parameter → `const T *`
#[unsafe(no_mangle)]
pub extern "C" fn ref_maybe_uninit(x: &MaybeUninit<Point>) {}

/// `ManuallyDrop<MaybeUninit<T>>` → `T` (both transforms compose)
#[unsafe(no_mangle)]
pub extern "C" fn nested_manually_drop_maybe_uninit(x: ManuallyDrop<MaybeUninit<Point>>) {}

/// `Option<MaybeUninit<NonNull<Point>>>` must NOT be simplified via NPO.
/// `MaybeUninit` disables niche optimization even though `NonNull` is NPO-eligible.
#[unsafe(no_mangle)]
pub extern "C" fn option_maybe_uninit_nonnull(
    x: Option<MaybeUninit<NonNull<Point>>>,
) {}

/// `Option<NonNull<MaybeUninit<Point>>>` SHOULD simplify to `*mut T`.
/// NonNull provides its own niche (non-null pointer), so MaybeUninit nested inside
/// does not prevent NPO.
#[unsafe(no_mangle)]
pub extern "C" fn option_nonnull_maybe_uninit(
    x: Option<NonNull<MaybeUninit<Point>>>,
) {}
