//! Dep crate: exercises type, field, and variant renames across headers.

#[repr(C)]
#[cheadergen::config(rename = "Point")]
pub struct InternalPoint {
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
#[cheadergen::config(export)]
pub struct Dimensions {
    #[cheadergen(rename = "w")]
    pub width: i32,
    #[cheadergen(rename = "h")]
    pub height: i32,
}

#[repr(C)]
#[cheadergen::config(export)]
pub enum Color {
    Red,
    #[cheadergen(rename = "COLOR_GREEN")]
    Green,
    Blue,
}

#[repr(C)]
#[cheadergen::config(export)]
pub enum Shape {
    #[cheadergen(rename = "SHAPE_CIRCLE")]
    Circle(f32),
    Rect { w: i32, h: i32 },
}
