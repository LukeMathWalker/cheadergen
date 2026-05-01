#[cheadergen::config(prefix_with_name, rename_all = "PascalCase")]
#[repr(C)]
pub enum Status {
    Ok,
    Error,
}

fn main() {}
