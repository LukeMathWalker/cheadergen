#[cheadergen::config(export)]
#[repr(C)]
pub struct Config {
    #[cheadergen(rename = "raw_width")]
    pub width: u32,
    pub height: u32,
}

fn main() {}
