#[repr(C)]
pub struct BetaEarly {
    pub value: u8,
}

#[unsafe(no_mangle)]
pub extern "C" fn beta_early() -> BetaEarly {
    BetaEarly { value: 0 }
}
