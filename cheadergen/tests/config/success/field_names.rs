#[cheadergen::config(field_names(x, y))]
#[repr(C)]
pub struct Point2D(f64, f64);

fn main() {}
