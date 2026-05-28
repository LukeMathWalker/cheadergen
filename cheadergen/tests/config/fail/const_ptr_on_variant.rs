#[cheadergen::config(export)]
#[repr(C)]
pub enum Status {
    #[cheadergen(const_ptr)]
    Ready,
}

fn main() {}
