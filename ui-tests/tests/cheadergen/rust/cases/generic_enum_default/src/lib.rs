//! Generic `#[repr(C)]` enum with a default type parameter.
//! `MaybeError<T, E = u8>` is used both relying on the default
//! (`MaybeError<i32>`) and overriding it (`MaybeError<i32, u16>`).

#[repr(C)]
pub enum MaybeError<T, E = u8> {
    Ok(T),
    Err(E),
}

#[unsafe(no_mangle)]
pub extern "C" fn use_default(m: MaybeError<i32>) -> MaybeError<i32> {
    m
}

#[unsafe(no_mangle)]
pub extern "C" fn use_override(m: MaybeError<i32, u16>) -> MaybeError<i32, u16> {
    m
}
