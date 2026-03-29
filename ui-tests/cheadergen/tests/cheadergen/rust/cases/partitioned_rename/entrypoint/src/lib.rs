//! Target crate: uses types with renamed fields, variants, and type names
//! across the header boundary.

use partitioned_rename_dep::{Color, Dimensions, InternalPoint, Shape};

#[unsafe(no_mangle)]
pub extern "C" fn use_point(p: InternalPoint) -> i32 {
    p.x + p.y
}

#[unsafe(no_mangle)]
pub extern "C" fn use_dims(d: Dimensions) -> i32 {
    d.width + d.height
}

#[unsafe(no_mangle)]
pub extern "C" fn use_color(c: Color) -> i32 {
    c as i32
}

#[unsafe(no_mangle)]
pub extern "C" fn use_shape(s: Shape) -> f32 {
    match s {
        Shape::Circle(r) => r,
        Shape::Rect { w, h } => (w + h) as f32,
    }
}
