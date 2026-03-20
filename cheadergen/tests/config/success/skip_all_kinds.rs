#[cheadergen::config(skip)]
#[repr(C)]
pub struct MyStruct {
    pub x: u32,
}

#[cheadergen::config(skip)]
#[repr(C)]
pub enum MyEnum {
    A,
    B,
}

#[cheadergen::config(skip)]
#[repr(C)]
pub union MyUnion {
    pub a: u32,
    pub b: f32,
}

#[cheadergen::config(skip)]
pub type MyAlias = u32;

#[cheadergen::config(skip)]
#[unsafe(no_mangle)]
pub extern "C" fn my_func() {}

#[cheadergen::config(skip)]
#[unsafe(no_mangle)]
pub static GLOBAL: u32 = 42;

fn main() {}
