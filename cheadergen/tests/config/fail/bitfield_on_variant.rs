#[cheadergen::config(export)]
#[repr(C)]
pub enum Status {
    #[cheadergen(bitfield = 8)]
    Ok,
    Error,
}

fn main() {}
