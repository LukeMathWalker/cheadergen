//! Test that `enumflags2::BitFlags<T>` (a `#[repr(transparent)]` wrapper
//! around the enum's underlying integer type) is correctly simplified
//! in function signatures.

use enumflags2::{bitflags, BitFlags};

#[bitflags]
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MyFlag {
    A = 0b0001,
    B = 0b0010,
}

#[unsafe(no_mangle)]
pub extern "C" fn accepts_flags(flags: BitFlags<MyFlag>) -> BitFlags<MyFlag> {
    flags
}
