//! Structs containing zero-sized type fields (`PhantomData`, `PhantomPinned`, `()`).
//!
//! Tests that cheadergen correctly omits ZST fields from the generated C struct.

use std::marker::{PhantomData, PhantomPinned};

#[repr(C)]
pub struct WithPhantomData {
    pub value: u32,
    pub _phantom: PhantomData<u64>,
}

#[repr(C)]
pub struct WithPhantomPinned {
    pub value: u32,
    pub _pin: PhantomPinned,
}

#[repr(C)]
pub struct WithUnitField {
    pub value: u32,
    pub _unit: (),
}

#[unsafe(no_mangle)]
pub extern "C" fn create_with_phantom_data() -> WithPhantomData {
    WithPhantomData {
        value: 0,
        _phantom: PhantomData,
    }
}

#[unsafe(no_mangle)]
#[expect(improper_ctypes_definitions)]
pub extern "C" fn create_with_phantom_pinned() -> WithPhantomPinned {
    WithPhantomPinned {
        value: 0,
        _pin: PhantomPinned,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn create_with_unit() -> WithUnitField {
    WithUnitField {
        value: 0,
        _unit: (),
    }
}
