#[cheadergen::config(rename_all_fields = "camelCase")]
#[unsafe(no_mangle)]
pub extern "C" fn my_func() {}

fn main() {}
