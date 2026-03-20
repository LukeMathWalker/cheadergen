#[cheadergen::config(prefix_with_name = false)]
#[repr(C)]
pub enum Status {
    Ok,
    Error,
}

fn main() {}
