#[cheadergen::config(export)]
#[repr(C)]
pub struct Flags {
    #[cheadergen(bitfield = 8)]
    pub flags: u8,
    pub value: u32,
}

fn main() {}
