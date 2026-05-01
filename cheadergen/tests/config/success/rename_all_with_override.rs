#[cheadergen::config(rename_all = "SCREAMING_SNAKE_CASE")]
#[repr(C)]
pub enum Color {
    Red,
    #[cheadergen(rename = "explicit_green")]
    Green,
    Blue,
}

fn main() {}
