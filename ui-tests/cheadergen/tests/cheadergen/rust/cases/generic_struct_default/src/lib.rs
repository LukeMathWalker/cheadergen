//! Generic `#[repr(C)]` structs with default type parameters.
//! `WithDefault<T, U = f32>` is used both relying on the default
//! (`WithDefault<i32>`) and overriding it (`WithDefault<i32, u8>`).

#[repr(C)]
pub struct WithDefault<T, U = f32> {
    pub main: T,
    pub extra: U,
}

#[unsafe(no_mangle)]
pub extern "C" fn use_default(w: WithDefault<i32>) -> WithDefault<i32> {
    w
}

#[unsafe(no_mangle)]
pub extern "C" fn use_override(w: WithDefault<i32, u8>) -> WithDefault<i32, u8> {
    w
}
