//! Comprehensive tests for `#[repr(transparent)]` structs.
//!
//! Covers tuple structs, named-field structs, ZST filtering,
//! pointer wrapping, chained transparent types, and usage in
//! other `repr(C)` structs and behind pointers.

use std::marker::{PhantomData, PhantomPinned};

// --- Inner types used by transparent wrappers ---

#[repr(C)]
pub struct Inner {
    pub x: f64,
    pub y: f64,
}

// --- 1. Tuple struct wrapping a primitive ---

#[repr(transparent)]
pub struct TransparentPrimitive(pub u32);

// --- 2. Tuple struct wrapping a repr(C) struct ---

#[repr(transparent)]
pub struct TransparentStruct(pub Inner);

// --- 3. Named-field struct wrapping a primitive ---

#[repr(transparent)]
pub struct TransparentNamedPrimitive {
    pub field: u32,
}

// --- 4. Named-field struct wrapping a repr(C) struct ---

#[repr(transparent)]
pub struct TransparentNamedStruct {
    pub field: Inner,
}

// --- 5. Struct with PhantomData + real field ---

#[repr(transparent)]
pub struct TransparentWithPhantomData {
    pub value: u32,
    pub _phantom: PhantomData<u64>,
}

// --- 6. Struct with PhantomPinned + real field ---

#[repr(transparent)]
pub struct TransparentWithPhantomPinned {
    pub value: u32,
    pub _pin: PhantomPinned,
}

// --- 7. Struct with () unit field + real field ---

#[repr(transparent)]
pub struct TransparentWithUnit {
    pub value: u32,
    pub _unit: (),
}

// --- 8. Empty/unit transparent struct (no non-ZST fields) ---

#[repr(transparent)]
pub struct TransparentEmpty {
    pub _phantom: PhantomData<u32>,
}

// --- 9. Transparent wrapping a pointer type ---

#[repr(transparent)]
pub struct TransparentPointer(pub *const u32);

#[repr(transparent)]
pub struct TransparentMutPointer(pub *mut Inner);

// --- 10. Transparent wrapping another transparent ---

#[repr(transparent)]
pub struct TransparentChained(pub TransparentPrimitive);

// --- 11 & 12. Transparent used behind a pointer and by value in repr(C) struct ---

#[repr(C)]
pub struct UsesTransparent {
    pub by_value: TransparentPrimitive,
    pub by_pointer: *const TransparentStruct,
}

// --- Root function to ensure all types appear in the header ---

#[unsafe(no_mangle)]
pub extern "C" fn root(
    _a: TransparentPrimitive,
    _b: TransparentStruct,
    _c: TransparentNamedPrimitive,
    _d: TransparentNamedStruct,
    _e: TransparentWithPhantomData,
    _f: TransparentWithPhantomPinned,
    _g: TransparentWithUnit,
    _h: TransparentEmpty,
    _i: TransparentPointer,
    _j: TransparentMutPointer,
    _k: TransparentChained,
    _l: UsesTransparent,
) {
}
