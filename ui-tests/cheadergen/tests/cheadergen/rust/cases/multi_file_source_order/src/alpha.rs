#[repr(C)]
pub struct AlphaEarly {
    pub value: u32,
}

#[unsafe(no_mangle)]
pub extern "C" fn alpha_early() -> AlphaEarly {
    AlphaEarly { value: 0 }
}

// Padding comments push the following items to line numbers larger than
// anything declared in `beta.rs`, so a (line, column)-only sort would
// interleave the two files.
//
//
//
//
//
//
//
//

#[repr(C)]
pub struct AlphaLate {
    pub value: u64,
}

#[unsafe(no_mangle)]
pub extern "C" fn alpha_late() -> AlphaLate {
    AlphaLate { value: 0 }
}
