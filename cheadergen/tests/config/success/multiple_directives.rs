#[cheadergen::config(export, rename = "CConfig")]
#[repr(C)]
pub struct Config {
    pub width: u32,
    pub height: u32,
}

fn main() {}
