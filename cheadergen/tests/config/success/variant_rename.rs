#[cheadergen::config(export)]
#[repr(C)]
pub enum Color {
    Red,
    #[cheadergen(rename = "COLOR_GREEN")]
    Green,
    Blue,
}

fn main() {}
