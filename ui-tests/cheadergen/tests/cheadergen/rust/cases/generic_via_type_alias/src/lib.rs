//! Regression test: a generic struct used via a type alias must be emitted
//! with its full definition when the alias appears as a by-value field.
//!
//! Previously, `collect_by_value_names_from_type` pushed only the alias's own
//! C name (e.g. `SmallVec_Foo`) for `Type::TypeAlias`, so the lookup in
//! `generic_by_name` (keyed by the underlying monomorphization, e.g.
//! `Vec2_Foo__u16`) missed and the generic was emitted as a forward
//! declaration — producing invalid C for a by-value field.

#[repr(C)]
pub struct Vec2<T, const N: u16> {
    pub data: *mut T,
    pub len: u16,
    pub cap: u16,
}

pub type SmallVec<T> = Vec2<T, 8>;

#[repr(C)]
pub struct Foo {
    pub value: u32,
}

#[repr(C)]
pub struct Holder {
    pub items: SmallVec<Foo>,
    pub count: u32,
}

#[unsafe(no_mangle)]
pub extern "C" fn take_holder(_h: Holder) {}
