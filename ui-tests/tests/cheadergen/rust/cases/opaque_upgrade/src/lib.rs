//! The generated header must include a struct definition for `Inner`,
//! since it appears both behind a pointer ([`Outer::ptr`]) and
//! bare ([`Outer::nested`] -> [`Middle::inner`]).

#[repr(C)]
pub struct Inner {
    pub value: i32,
}

#[repr(C)]
pub struct Middle {
    pub inner: Inner,
}

#[repr(C)]
pub struct Outer {
    pub ptr: *const Inner,
    pub nested: Middle,
}

#[unsafe(no_mangle)]
pub extern "C" fn root(o: Outer) {}
