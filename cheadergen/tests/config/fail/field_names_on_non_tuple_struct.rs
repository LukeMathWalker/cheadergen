#[cheadergen::config(field_names(x, y))]
#[repr(C)]
pub struct Named {
    pub a: f64,
    pub b: f64,
}

#[cheadergen::config(field_names(x))]
pub struct Unit;

#[cheadergen::config(field_names(x, y))]
#[repr(C)]
pub enum Status {
    Ok,
    Error,
}

#[cheadergen::config(field_names(x, y))]
#[repr(C)]
pub union MyUnion {
    pub a: f64,
    pub b: f64,
}

#[cheadergen::config(field_names(x))]
#[unsafe(no_mangle)]
pub extern "C" fn my_func() {}

#[cheadergen::config(field_names(x))]
#[unsafe(no_mangle)]
pub static GLOBAL: u32 = 42;

fn main() {}
