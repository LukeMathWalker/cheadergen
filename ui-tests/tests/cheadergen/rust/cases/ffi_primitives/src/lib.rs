//! `std::ffi` primitive type aliases used in `extern "C"` signatures.
//!
//! These types (`c_int`, `c_char`, `c_double`, etc.) map directly to C types
//! and should ideally not produce redundant typedefs in the generated header.

use std::ffi::{
    c_char, c_double, c_float, c_int, c_long, c_longlong, c_schar, c_short, c_uchar, c_uint,
    c_ulong, c_ulonglong, c_ushort, c_void,
};

// --- One function per integer type ---

#[unsafe(no_mangle)]
pub extern "C" fn accept_char(x: c_char) -> c_char {
    x
}

#[unsafe(no_mangle)]
pub extern "C" fn accept_schar(x: c_schar) -> c_schar {
    x
}

#[unsafe(no_mangle)]
pub extern "C" fn accept_uchar(x: c_uchar) -> c_uchar {
    x
}

#[unsafe(no_mangle)]
pub extern "C" fn accept_short(x: c_short) -> c_short {
    x
}

#[unsafe(no_mangle)]
pub extern "C" fn accept_ushort(x: c_ushort) -> c_ushort {
    x
}

#[unsafe(no_mangle)]
pub extern "C" fn accept_int(x: c_int) -> c_int {
    x
}

#[unsafe(no_mangle)]
pub extern "C" fn accept_uint(x: c_uint) -> c_uint {
    x
}

#[unsafe(no_mangle)]
pub extern "C" fn accept_long(x: c_long) -> c_long {
    x
}

#[unsafe(no_mangle)]
pub extern "C" fn accept_ulong(x: c_ulong) -> c_ulong {
    x
}

#[unsafe(no_mangle)]
pub extern "C" fn accept_longlong(x: c_longlong) -> c_longlong {
    x
}

#[unsafe(no_mangle)]
pub extern "C" fn accept_ulonglong(x: c_ulonglong) -> c_ulonglong {
    x
}

// --- Float types ---

#[unsafe(no_mangle)]
pub extern "C" fn accept_float(x: c_float) -> c_float {
    x
}

#[unsafe(no_mangle)]
pub extern "C" fn accept_double(x: c_double) -> c_double {
    x
}

// --- Void pointers ---

#[unsafe(no_mangle)]
pub extern "C" fn accept_void_ptr(x: *mut c_void) -> *mut c_void {
    x
}

#[unsafe(no_mangle)]
pub extern "C" fn accept_const_void_ptr(x: *const c_void) -> *const c_void {
    x
}

// --- Struct with mixed ffi-type fields ---

#[repr(C)]
pub struct MixedFfi {
    pub character: c_char,
    pub count: c_int,
    pub size: c_ulong,
    pub ratio: c_double,
    pub data: *mut c_void,
}

#[unsafe(no_mangle)]
pub extern "C" fn accept_mixed(x: MixedFfi) -> MixedFfi {
    x
}
