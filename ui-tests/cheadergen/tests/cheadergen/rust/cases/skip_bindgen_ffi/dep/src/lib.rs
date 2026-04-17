//! Simulates a bindgen-generated FFI crate: a mix of a type alias,
//! a `#[repr(C)]` struct, and a `#[repr(C)]` enum, all of which the
//! consumer treats as externally-defined and references by bare name.

pub type size_t = usize;

#[repr(C)]
pub struct sockaddr {
    pub family: u16,
    pub data: [u8; 14],
}

#[repr(C)]
pub enum log_level {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}
