//! A type reachable through two paths (inner module + `pub use` at the crate root)
//! must produce exactly one C typedef, not one per path.

mod inner {
    #[repr(C)]
    pub struct Point {
        pub x: f64,
        pub y: f64,
    }
}

pub use inner::Point;

#[unsafe(no_mangle)]
pub extern "C" fn point_origin() -> Point {
    Point { x: 0.0, y: 0.0 }
}

#[repr(C)]
pub struct Pair {
    pub a: inner::Point,
    pub b: inner::Point,
}

#[unsafe(no_mangle)]
pub extern "C" fn pair_new(a: inner::Point, b: inner::Point) -> Pair {
    Pair { a, b }
}
