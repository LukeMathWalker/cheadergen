#[cheadergen::config(rename_all_fields = "camelCase")]
#[unsafe(no_mangle)]
pub static GLOBAL: u32 = 42;

fn main() {}
