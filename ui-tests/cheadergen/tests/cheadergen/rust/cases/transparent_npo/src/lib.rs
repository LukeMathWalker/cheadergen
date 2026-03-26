//! Null pointer optimization for `#[repr(transparent)]` wrappers.
//!
//! Rust guarantees NPO for `Option<T>` when `T` is a `#[repr(transparent)]`
//! struct wrapping an NPO-eligible type (references, `Box`, `NonNull`, fn pointers),
//! recursively through chains of transparent wrappers.
//!
//! `Option<TransparentWrapper>` should simplify to `TransparentWrapper` in the header,
//! since the typedef already resolves to a pointer-like type and the Option just uses
//! null as the niche.
//!
//! Note: wrapping `NonNull<T>` or `Box<T>` inside a `#[repr(transparent)]` struct is
//! not yet supported because cross-crate type resolution cannot look up those std types.
//! Direct `Option<NonNull<T>>` and `Option<Box<T>>` in function signatures ARE handled
//! (see the `nonnull_pointers` and `box_pointers` test cases).

// --- Transparent wrapper around a reference (NPO-eligible) ---

#[repr(transparent)]
pub struct MyRef<'a>(pub &'a i32);

// --- Chained: transparent wrapping another transparent (recursive NPO) ---

#[repr(transparent)]
pub struct ChainedRef<'a>(pub MyRef<'a>);

// --- Transparent wrapper around a function pointer ---

#[repr(transparent)]
pub struct MyCb(pub extern "C" fn(i32) -> i32);

// --- Chained through a function pointer wrapper ---

#[repr(transparent)]
pub struct ChainedCb(pub MyCb);

// --- Functions exercising Option<Wrapper> (should simplify via NPO) ---

#[unsafe(no_mangle)]
pub extern "C" fn take_option_myref(x: Option<MyRef<'_>>) {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn take_option_chained_ref(x: Option<ChainedRef<'_>>) {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn take_option_mycb(x: Option<MyCb>) {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn take_option_chained_cb(x: Option<ChainedCb>) {
    todo!()
}

// --- Bare usage without Option (typedef should still work normally) ---

#[unsafe(no_mangle)]
pub extern "C" fn take_bare_myref(x: MyRef<'_>) {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn take_bare_chained_ref(x: ChainedRef<'_>) {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn take_bare_mycb(x: MyCb) {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn take_bare_chained_cb(x: ChainedCb) {
    todo!()
}
