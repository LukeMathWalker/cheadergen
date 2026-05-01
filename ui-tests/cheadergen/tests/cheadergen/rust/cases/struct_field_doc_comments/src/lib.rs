//! Doc comments on struct fields must be preserved in the generated header,
//! for both named-field structs, tuple structs, and union fields.

#[repr(C)]
pub struct Point {
    /// The horizontal coordinate.
    pub x: f32,
    /// The vertical coordinate.
    pub y: f32,
}

#[repr(C)]
pub struct Pair(
    /// The first element.
    pub i32,
    /// The second element.
    pub u32,
);

#[repr(C)]
pub union IntOrFloat {
    /// Interpret the bits as an unsigned integer.
    pub u: u32,
    /// Interpret the bits as a float.
    pub f: f32,
}

#[unsafe(no_mangle)]
pub extern "C" fn root(point: Point, pair: Pair, value: IntOrFloat) {}
