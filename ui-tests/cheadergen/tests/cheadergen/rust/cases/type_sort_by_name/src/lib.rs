//! `sort_by = "name"` must order type definitions by their C name instead
//! of source position — declarations here are deliberately anti-alphabetical.
//!
//! The topological constraint still dominates: `Middle` contains `ZInner`
//! by value, so `ZInner` is emitted before `Middle` even though `Middle`
//! sorts first by name.

#[repr(C)]
pub struct Zebra {
    pub value: u32,
}

#[repr(C)]
pub enum ZooKind {
    Zoo,
}

#[repr(C)]
pub struct ZInner {
    pub value: u8,
}

#[repr(C)]
pub struct Middle {
    pub inner: ZInner,
}

#[repr(C)]
pub enum AnimalKind {
    Animal,
}

#[repr(C)]
pub struct Apple {
    pub value: u64,
}

#[unsafe(no_mangle)]
pub extern "C" fn zebra_new() -> Zebra {
    Zebra { value: 0 }
}

#[unsafe(no_mangle)]
pub extern "C" fn zoo_kind() -> ZooKind {
    ZooKind::Zoo
}

#[unsafe(no_mangle)]
pub extern "C" fn middle_new() -> Middle {
    Middle {
        inner: ZInner { value: 0 },
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn animal_kind() -> AnimalKind {
    AnimalKind::Animal
}

#[unsafe(no_mangle)]
pub extern "C" fn apple_new() -> Apple {
    Apple { value: 0 }
}
