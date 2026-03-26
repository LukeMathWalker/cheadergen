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

#[repr(u16)]
pub enum U16Enum {
    U16A,
    U16B,
}

#[repr(u32)]
pub enum U32Enum {
    U32A,
    U32B,
}

#[repr(u64)]
pub enum U64Enum {
    U64A,
    U64B,
}

#[repr(u128)]
pub enum U128Enum {
    U128A,
    U128B,
}

#[repr(i16)]
pub enum I16Enum {
    I16A,
    I16B,
}

#[repr(i32)]
pub enum I32Enum {
    I32A,
    I32B,
}

#[repr(i64)]
pub enum I64Enum {
    I64A,
    I64B,
}

#[repr(i128)]
pub enum I128Enum {
    I128A,
    I128B,
}

#[repr(isize)]
pub enum IsizeEnum {
    IsizeA,
    IsizeB,
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
    u16_enum: U16Enum,
    u32_enum: U32Enum,
    u64_enum: U64Enum,
    u128_enum: U128Enum,
    i16_enum: I16Enum,
    i32_enum: I32Enum,
    i64_enum: I64Enum,
    i128_enum: I128Enum,
    isize_enum: IsizeEnum,
    no_repr: *const NoRepr,
) {
}
