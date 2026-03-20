#[cheadergen::config(rename = "CMyStruct")]
#[repr(C)]
pub struct MyStruct {
    pub x: u32,
}

#[cheadergen::config(rename = "CMyEnum")]
#[repr(C)]
pub enum MyEnum {
    A,
    B,
}

#[cheadergen::config(rename = "CMyUnion")]
#[repr(C)]
pub union MyUnion {
    pub a: u32,
    pub b: f32,
}

#[cheadergen::config(rename = "CMyAlias")]
pub type MyAlias = u32;

#[cheadergen::config(rename = "c_my_func")]
#[unsafe(no_mangle)]
pub extern "C" fn my_func() {}

#[cheadergen::config(rename = "C_GLOBAL")]
#[unsafe(no_mangle)]
pub static GLOBAL: u32 = 42;

fn main() {}
