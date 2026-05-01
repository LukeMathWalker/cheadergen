#[cheadergen::config(rename_all = "snake_case")]
#[unsafe(no_mangle)]
pub static GLOBAL: u32 = 42;

fn main() {}
