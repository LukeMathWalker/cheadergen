//! Tagged unions (enums with data variants) under various repr attributes.
//!
//! Tests that cheadergen emits correct C declarations for:
//! - `#[repr(C)]` tagged unions (struct with tag enum + anonymous union)
//! - `#[repr(u8)]` tagged unions (union with tag + anonymous struct members)
//! - `#[repr(C, u8)]` tagged unions (like repr(C) but tag has int typedef)
//! - Unit, single-field tuple, and multi-field struct variants

#[repr(u8)]
pub enum Untagged {
    Foo(i16),
    Bar { x: u8, y: i16 },
    Baz,
}

#[repr(C)]
pub enum Tagged {
    Foo(i16),
    Bar { x: u8, y: i16 },
    Baz,
}

#[repr(C, u8)]
pub enum TaggedInt {
    Foo(i16),
    Bar { x: u8, y: i16 },
    Baz,
}

#[repr(u16)]
pub enum UntaggedU16 {
    U16Data(i16),
    U16Unit,
}

#[repr(u32)]
pub enum UntaggedU32 {
    U32Data(i16),
    U32Unit,
}

#[repr(u64)]
pub enum UntaggedU64 {
    U64Data(i16),
    U64Unit,
}

#[repr(u128)]
pub enum UntaggedU128 {
    U128Data(i16),
    U128Unit,
}

#[repr(i8)]
pub enum UntaggedI8 {
    I8Data(i16),
    I8Unit,
}

#[repr(i16)]
pub enum UntaggedI16 {
    I16Data(i16),
    I16Unit,
}

#[repr(i32)]
pub enum UntaggedI32 {
    I32Data(i16),
    I32Unit,
}

#[repr(i64)]
pub enum UntaggedI64 {
    I64Data(i16),
    I64Unit,
}

#[repr(i128)]
pub enum UntaggedI128 {
    I128Data(i16),
    I128Unit,
}

#[repr(isize)]
pub enum UntaggedIsize {
    IsizeData(i16),
    IsizeUnit,
}

#[unsafe(no_mangle)]
pub extern "C" fn root(
    a: Untagged,
    b: Tagged,
    c: TaggedInt,
    d: UntaggedU16,
    e: UntaggedU32,
    f: UntaggedU64,
    g: UntaggedU128,
    h: UntaggedI8,
    i: UntaggedI16,
    j: UntaggedI32,
    k: UntaggedI64,
    l: UntaggedI128,
    m: UntaggedIsize,
) {
}
