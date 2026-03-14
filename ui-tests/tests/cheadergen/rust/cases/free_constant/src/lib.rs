//! Free-standing primitive constants emitted as `#define` macros.

/// The maximum buffer size.
pub const MAX_SIZE: usize = 1024;

pub const MIN_VALUE: i32 = -42;

pub const PI_APPROX: f64 = 3.14159;

pub const IS_ENABLED: bool = true;

pub const BYTE_MASK: u8 = 0xFF;

pub const BIG_NUMBER: u64 = 1_000_000;

pub const SMALL_UNSIGNED: u16 = 256;

pub const MEDIUM_UNSIGNED: u32 = 100_000;

pub const TINY_SIGNED: i8 = -1;

pub const SMALL_SIGNED: i16 = -1000;

pub const BIG_SIGNED: i64 = -9_000_000_000;

pub const PTR_OFFSET: isize = -8;

pub const HALF_PRECISION: f32 = 2.5;

/// This function uses MAX_SIZE to demonstrate constants alongside functions.
#[unsafe(no_mangle)]
pub extern "C" fn get_max_size() -> usize {
    MAX_SIZE
}
