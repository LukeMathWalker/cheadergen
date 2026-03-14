//! Rust `type` aliases emitted as C `typedef`s.
//!
//! Covers aliases to primitives, structs, enums, and unions,
//! both one layer deep (`type A = Concrete`) and two layers deep
//! (`type B = A` where `A` is itself an alias). Also checks that
//! aliases are usable in function signatures and struct fields.

// --- Concrete types that aliases will point to ---

#[repr(C)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[repr(C)]
pub enum Color {
    Red,
    Green,
    Blue,
}

#[repr(C)]
pub union FloatOrInt {
    pub f: f32,
    pub i: i32,
}

// --- 1-layer aliases ---

/// Alias to a primitive.
pub type Length = u32;

/// Alias to a repr(C) struct.
pub type Position = Point;

/// Alias to a repr(C) enum.
pub type Shade = Color;

/// Alias to a repr(C) union.
pub type Number = FloatOrInt;

// --- 2-layer aliases (alias → alias → concrete) ---

/// Alias to an alias to a primitive.
pub type Size = Length;

/// Alias to an alias to a struct.
pub type Coordinate = Position;

// --- Usage in a struct field ---

#[repr(C)]
pub struct Rect {
    pub origin: Position,
    pub width: Length,
    pub height: Length,
}

// --- Root function exercising all aliases in signatures ---

#[unsafe(no_mangle)]
pub extern "C" fn use_aliases(
    _len: Length,
    _pos: Position,
    _shade: Shade,
    _num: Number,
    _size: Size,
    _coord: Coordinate,
    _rect: Rect,
) {
}
