#[cheadergen::config(bitfield = 8)]
#[repr(C)]
pub struct Flags {
    pub flags: u8,
}

fn main() {}
