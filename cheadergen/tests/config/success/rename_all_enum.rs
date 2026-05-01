#[cheadergen::config(rename_all = "SCREAMING_SNAKE_CASE")]
#[repr(C)]
pub enum Status {
    Ok,
    Error,
    Pending,
}

fn main() {}
