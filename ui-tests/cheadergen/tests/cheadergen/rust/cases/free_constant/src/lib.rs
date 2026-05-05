//! Free-standing primitive constants emitted as `#define` macros.
//!
//! Constants are opt-in: only those carrying `#[cheadergen::config(export)]`
//! reach the generated header.

/// The maximum buffer size.
#[cheadergen::config(export)]
pub const MAX_SIZE: usize = 1024;

#[cheadergen::config(export)]
pub const MIN_VALUE: i32 = -42;

#[cheadergen::config(export)]
pub const PI_APPROX: f64 = 3.14159;

#[cheadergen::config(export)]
pub const IS_ENABLED: bool = true;

#[cheadergen::config(export)]
pub const BYTE_MASK: u8 = 0xFF;

#[cheadergen::config(export)]
pub const BIG_NUMBER: u64 = 1_000_000;

#[cheadergen::config(export)]
pub const SMALL_UNSIGNED: u16 = 256;

#[cheadergen::config(export)]
pub const MEDIUM_UNSIGNED: u32 = 100_000;

#[cheadergen::config(export)]
pub const TINY_SIGNED: i8 = -1;

#[cheadergen::config(export)]
pub const SMALL_SIGNED: i16 = -1000;

#[cheadergen::config(export)]
pub const BIG_SIGNED: i64 = -9_000_000_000;

#[cheadergen::config(export)]
pub const PTR_OFFSET: isize = -8;

#[cheadergen::config(export)]
pub const HALF_PRECISION: f32 = 2.5;

/// This function uses MAX_SIZE to demonstrate constants alongside functions.
#[unsafe(no_mangle)]
pub extern "C" fn get_max_size() -> usize {
    MAX_SIZE
}
