//! Tagged unions whose variants carry explicit discriminants.
//!
//! Mirrors a real-world bitflag-style layout: a `#[repr(u8)]` tagged union
//! with non-sequential discriminant values (powers of two). The generated
//! tag enum must preserve those values verbatim so C-side code observes
//! the same numeric tags as Rust.

#[repr(u8)]
pub enum Bitflagged {
    Union = 1,
    Intersection = 2,
    Term = 4,
    Virtual = 8,
    Numeric = 16,
    Metric = 32,
    HybridMetric = 64,
}

#[repr(u8)]
pub enum BitflaggedWithData {
    Unit = 1,
    Single(u32) = 2,
    Pair { x: u8, y: u16 } = 4,
}

#[repr(C, u8)]
pub enum CTaggedExplicit {
    A = 10,
    B(u32) = 20,
    C { x: u16 } = 30,
}

#[unsafe(no_mangle)]
pub extern "C" fn root(a: Bitflagged, b: BitflaggedWithData, c: CTaggedExplicit) {
    let _ = (a, b, c);
}
