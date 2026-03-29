//! Dep crate: defines a tagged union (enum with data).

#[repr(C)]
pub enum Message {
    Text(i32),
    Binary { data: u32, len: u32 },
    Empty,
}
