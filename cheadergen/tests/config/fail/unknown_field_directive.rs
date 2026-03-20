#[cheadergen::config(export)]
#[repr(C)]
pub struct Config {
    #[cheadergen(exprot = "x")]
    pub width: u32,
}

fn main() {}
