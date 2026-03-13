//! Tagged unions generic over lifetime parameters. Lifetime parameters
//! must be stripped in C output (C has no concept of lifetimes), and
//! references become pointers. Mixed lifetime + type parameter enums
//! are monomorphized by type parameters only.

#[repr(C)]
pub enum Ref<'a> {
    Borrowed(&'a i32),
    Owned(i32),
}

#[repr(C)]
pub enum MutOrConst<'a, 'b> {
    Shared(&'a u8),
    Exclusive(&'b mut u8),
}

#[repr(C)]
pub enum RefOrVal<'a, T> {
    Ref(&'a T),
    Val(T),
}

#[unsafe(no_mangle)]
pub extern "C" fn use_ref(r: Ref<'_>) -> Ref<'_> {
    r
}

#[unsafe(no_mangle)]
pub extern "C" fn use_mut_or_const<'a, 'b>(m: MutOrConst<'a, 'b>) -> MutOrConst<'a, 'b> {
    m
}

#[unsafe(no_mangle)]
pub extern "C" fn use_ref_or_val_i32(r: RefOrVal<'_, i32>) -> RefOrVal<'_, i32> {
    r
}

#[unsafe(no_mangle)]
pub extern "C" fn use_ref_or_val_f64(r: RefOrVal<'_, f64>) -> RefOrVal<'_, f64> {
    r
}
