//! References to primitive types (`&u32`, `&mut u32`, `&f64`, `&bool`, `&()`).
//!
//! Tests that cheadergen maps Rust references to the appropriate C pointer types.

#[unsafe(no_mangle)]
pub extern "C" fn read_u32(x: &u32) -> u32 {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn write_u32(x: &mut u32, val: u32) {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn read_f64(x: &f64) -> f64 {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn read_bool(x: &bool) -> bool {
    todo!()
}

#[unsafe(no_mangle)]
pub extern "C" fn ref_to_unit(x: &()) {
    todo!()
}
