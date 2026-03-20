#[cheadergen::config(prefix_with_name)]
#[repr(C)]
pub enum Status {
    Ok,
    Error,
    Pending,
}

fn main() {}
