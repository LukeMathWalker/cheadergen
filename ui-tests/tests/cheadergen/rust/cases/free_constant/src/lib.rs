//! Free-standing primitive constants emitted as `#define` macros.

/// The maximum buffer size.
pub const MAX_SIZE: usize = 1024;

pub const MIN_VALUE: i32 = -42;

pub const PI_APPROX: f64 = 3.14159;

pub const IS_ENABLED: bool = true;

pub const BYTE_MASK: u8 = 0xFF;

pub const BIG_NUMBER: u64 = 1_000_000;

/// This function uses MAX_SIZE to demonstrate constants alongside functions.
#[unsafe(no_mangle)]
pub extern "C" fn get_max_size() -> usize {
    MAX_SIZE
}
