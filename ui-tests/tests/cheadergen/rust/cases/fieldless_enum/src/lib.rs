//! Fieldless (C-like) enums with various repr attributes.
//!
//! Tests that cheadergen emits correct C declarations for:
//! - `#[repr(C)]` enums (plain C enum)
//! - `#[repr(u8)]`, `#[repr(i8)]`, `#[repr(usize)]` etc. (enum + typedef)
//! - Explicit discriminant values
//! - Enums without a valid repr (should remain opaque)

#[repr(C)]
pub enum Color {
    Red,
    Green,
    Blue,
}

#[repr(u8)]
pub enum Direction {
    Up = 0,
    Down = 2,
    Left,
    Right = 5,
}

#[repr(i8)]
pub enum Sign {
    Negative = -1,
    Zero = 0,
    Positive = 1,
}

#[repr(usize)]
pub enum Big {
    First = 0,
    Second = 2,
    Third,
}

/// No repr — should be emitted as opaque.
pub enum NoRepr {
    X,
    Y,
}

#[unsafe(no_mangle)]
pub extern "C" fn root(
    color: Color,
    dir: Direction,
    sign: Sign,
    big: Big,
    no_repr: *const NoRepr,
) {
}
