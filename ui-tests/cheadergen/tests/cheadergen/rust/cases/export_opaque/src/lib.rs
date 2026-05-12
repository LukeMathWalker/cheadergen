//! `export, opaque` forces a type into the header as an opaque forward
//! declaration, even when it has `#[repr(C)]` and full field visibility.

/// A struct exported as opaque — only a forward declaration should appear,
/// not the full definition, despite `#[repr(C)]` and by-value usage.
#[cheadergen::config(export, opaque)]
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
#[cheadergen::config(export, opaque)]
pub struct OpaqueHandle {
    _inner: u64,
}

/// A struct that embeds an opaque type by value — the generated header won't
/// compile because `OpaqueConfig` is an incomplete type.
#[repr(C)]
pub struct ContainsOpaque {
    pub label: u32,
    pub config: OpaqueConfig,
}

/// Takes an opaque type by value — invalid C (incomplete type as parameter).
#[unsafe(no_mangle)]
pub extern "C" fn consume_opaque(config: OpaqueConfig) -> u32 {
    config.width + config.height
}

/// Takes a struct containing an opaque type by value.
#[unsafe(no_mangle)]
pub extern "C" fn consume_contains_opaque(container: ContainsOpaque) -> u32 {
    container.label + container.config.width
}

/// Returns an opaque type by value — invalid C (incomplete return type).
#[unsafe(no_mangle)]
pub extern "C" fn produce_opaque() -> OpaqueConfig {
    OpaqueConfig {
        width: 640,
        height: 480,
    }
}
