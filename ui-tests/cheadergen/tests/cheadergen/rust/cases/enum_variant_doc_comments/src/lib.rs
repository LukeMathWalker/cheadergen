//! Doc comments on enum variants must be preserved in the generated header,
//! for both fieldless enums (`#[repr(C)]` and primitive `#[repr(...)]`) and
//! tagged unions (`#[repr(C)]` and `#[repr(C, uN)]`).

#[repr(C)]
pub enum Color {
    /// The color red.
    Red,
    /// The color green.
    Green,
    /// The color blue.
    Blue,
}

#[repr(u8)]
pub enum Direction {
    /// Pointing up.
    Up = 0,
    /// Pointing down.
    Down = 2,
    /// Pointing left.
    Left,
    /// Pointing right.
    Right = 5,
}

#[repr(C)]
pub enum Shape {
    /// A circle with the given radius.
    Circle(f32),
    /// A rectangle with width and height.
    Rectangle {
        /// The horizontal extent.
        width: f32,
        /// The vertical extent.
        height: f32,
    },
    /// A point — no associated data.
    Point,
}

#[repr(C, u8)]
pub enum Event {
    /// A keystroke event with the key code.
    Key(u32),
    /// A click at the given coordinates.
    Click {
        /// The X coordinate of the click.
        x: i32,
        /// The Y coordinate of the click.
        y: i32,
    },
    /// A signal that the input ended.
    Eof,
}

#[unsafe(no_mangle)]
pub extern "C" fn root(color: Color, direction: Direction, shape: Shape, event: Event) {}
