#[cheadergen::config(prefix_with_name)]
#[repr(C)]
pub struct Named {
    pub x: u32,
}

#[cheadergen::config(prefix_with_name)]
#[repr(C)]
pub struct Tuple(pub u32);

#[cheadergen::config(prefix_with_name)]
pub struct Unit;

#[cheadergen::config(prefix_with_name)]
#[repr(C)]
pub union MyUnion {
    pub a: u32,
    pub b: f32,
}

#[cheadergen::config(prefix_with_name)]
#[unsafe(no_mangle)]
pub extern "C" fn my_func() {}

#[cheadergen::config(prefix_with_name)]
#[unsafe(no_mangle)]
pub static GLOBAL: u32 = 42;

#[cheadergen::config(prefix_with_name)]
pub type Alias = u32;

fn main() {}
