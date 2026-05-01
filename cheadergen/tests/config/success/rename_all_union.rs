#[cheadergen::config(rename_all = "snake_case")]
#[repr(C)]
pub union MyUnion {
    pub fooBar: u32,
    pub bazQux: f32,
}

fn main() {}
