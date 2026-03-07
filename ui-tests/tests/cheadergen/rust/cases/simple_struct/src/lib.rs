#[repr(C)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[unsafe(no_mangle)]
pub extern "C" fn point_new(x: f64, y: f64) -> Point {
    Point { x, y }
}
