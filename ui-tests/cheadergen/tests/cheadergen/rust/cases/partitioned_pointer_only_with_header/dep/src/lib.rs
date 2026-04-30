//! Dep crate exposing several types kept reachable by their own extern "C"
//! fns. The entrypoint references each of them only behind a pointer; the
//! generated entrypoint header should declare each referenced type inline
//! rather than `#include` this dep's header — pulling in the dep header
//! would drag in declarations the entrypoint never uses.

#[repr(C)]
pub struct Anchor {
    pub value: i32,
}

#[repr(C)]
pub struct OtherStruct {
    pub flag: u8,
}

#[repr(C)]
#[cheadergen::config(rename = "RenamedFoo")]
pub struct InternalFoo {
    pub value: u64,
}

pub type Aliased = u32;

#[unsafe(no_mangle)]
pub extern "C" fn anchor_value(a: Anchor) -> i32 {
    a.value
}

#[unsafe(no_mangle)]
pub extern "C" fn other_flag(o: OtherStruct) -> u8 {
    o.flag
}

#[unsafe(no_mangle)]
pub extern "C" fn foo_value(f: InternalFoo) -> u64 {
    f.value
}
