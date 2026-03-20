#[cheadergen::config(field_names(x, y, z))]
#[repr(C)]
pub struct Point2D(pub f64, pub f64);

fn main() {}
