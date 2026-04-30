//! Holds both a typedef and an unrelated struct.
//!
//! Because `Inner` is referenced by-value from another dep, this crate keeps
//! its own header (the opaque-only pruning does not apply). The `Aliased`
//! typedef therefore stays in this crate's header, not in the global
//! `opaque_types` pool.

#[repr(C)]
pub struct Inner {
    pub x: i32,
}

pub type Aliased = u32;
