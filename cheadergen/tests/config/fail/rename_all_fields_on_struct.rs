#[cheadergen::config(rename_all_fields = "camelCase")]
#[repr(C)]
pub struct Foo {
    pub field: u32,
}

fn main() {}
