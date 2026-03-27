//! `export(opaque)` forces a type into the header as an opaque forward
//! declaration, even when it has `#[repr(C)]` and full field visibility.

/// A struct exported as opaque — only a forward declaration should appear,
/// not the full definition, despite `#[repr(C)]` and by-value usage.
#[cheadergen::config(export(opaque))]
#[repr(C)]
pub struct OpaqueConfig {
    pub width: u32,
    pub height: u32,
}

/// A struct exported normally — full definition should appear.
#[cheadergen::config(export)]
#[repr(C)]
pub struct FullConfig {
    pub width: u32,
    pub height: u32,
}

/// A non-repr(C) type exported as opaque — should also produce a forward declaration.
#[cheadergen::config(export(opaque))]
pub struct OpaqueHandle {
    _inner: u64,
}
