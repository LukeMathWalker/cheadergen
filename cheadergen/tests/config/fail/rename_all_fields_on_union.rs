#[cheadergen::config(rename_all_fields = "camelCase")]
#[repr(C)]
pub union MyUnion {
    pub a: u32,
    pub b: f32,
}

fn main() {}
