#[cheadergen::config(rename_all = "camelCase")]
#[repr(C)]
pub struct Settings {
    pub max_value: u32,
    pub min_value: u32,
}

fn main() {}
