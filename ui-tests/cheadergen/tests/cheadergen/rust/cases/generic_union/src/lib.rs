//! Generic `#[repr(C)]` unions must be monomorphized into concrete C union
//! definitions. `Wrapper<i32>` and `Wrapper<f32>` produce separate typedefs,
//! and `Pair<T, U>` exercises multiple type parameters.

#[repr(C)]
pub union Wrapper<T: Copy> {
    pub value: T,
    pub tag: u32,
}

#[repr(C)]
pub union Pair<T: Copy, U: Copy> {
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
