//! Larger alignment value (`align(64)`) — a common pattern for cache-line
//! padding. Verifies the `N` is propagated as-is into the emitted macro.

#[repr(C, align(64))]
pub struct CacheLineAligned {
    pub counter: u64,
}

#[unsafe(no_mangle)]
pub extern "C" fn cache_line_aligned_new() -> CacheLineAligned {
    CacheLineAligned { counter: 0 }
}
