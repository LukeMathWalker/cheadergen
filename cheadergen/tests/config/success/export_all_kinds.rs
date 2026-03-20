#[cheadergen::config(export)]
#[repr(C)]
pub struct MyStruct {
    pub x: u32,
}

#[cheadergen::config(export)]
#[repr(C)]
pub enum MyEnum {
    A,
    B,
}

#[cheadergen::config(export)]
#[repr(C)]
pub union MyUnion {
    pub a: u32,
    pub b: f32,
}

#[cheadergen::config(export)]
pub type MyAlias = u32;

fn main() {}
