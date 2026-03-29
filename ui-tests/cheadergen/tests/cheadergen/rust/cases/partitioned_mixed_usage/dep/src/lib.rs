//! Dependency crate for `partitioned_mixed_usage`.

#[repr(C)]
pub struct DepType1 {
    pub a: i32,
}

#[repr(C)]
pub struct DepType2 {
    pub b: f64,
}
