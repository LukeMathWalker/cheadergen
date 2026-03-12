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

#[unsafe(no_mangle)]
pub extern "C" fn root(
    a: Untagged,
    b: Tagged,
    c: TaggedInt,
) {
}
