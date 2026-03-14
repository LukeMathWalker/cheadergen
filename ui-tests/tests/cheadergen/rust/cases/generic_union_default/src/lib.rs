//! Generic `#[repr(C)]` union with a default type parameter.
//! `WithDefault<T, U = f32>` is used both relying on the default
//! (`WithDefault<i32>`) and overriding it (`WithDefault<i32, u8>`).

#[repr(C)]
pub union WithDefault<T: Copy, U: Copy = f32> {
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
