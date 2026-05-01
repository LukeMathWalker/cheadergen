#[cheadergen::config(rename_all = "snake_case")]
#[unsafe(no_mangle)]
pub extern "C" fn my_func() {}

fn main() {}
