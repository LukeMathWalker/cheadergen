//! Generic `#[repr(C)]` structs must be monomorphized into concrete C struct
//! definitions. `Wrapper<i32>` and `Wrapper<f32>` produce separate typedefs,
//! and `Pair<T, U>` exercises multiple type parameters.

#[repr(C)]
pub struct Wrapper<T> {
    pub value: T,
}

#[repr(C)]
pub struct Pair<T, U> {
    pub first: T,
    pub second: U,
}

#[unsafe(no_mangle)]
pub extern "C" fn use_wrapper_i32(w: Wrapper<i32>) -> Wrapper<i32> {
    w
}

#[unsafe(no_mangle)]
pub extern "C" fn use_wrapper_f32(w: Wrapper<f32>) -> Wrapper<f32> {
    w
}

#[unsafe(no_mangle)]
pub extern "C" fn use_pair(p: Pair<i32, f32>) -> Pair<i32, f32> {
    p
}
