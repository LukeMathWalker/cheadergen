#[cheadergen::config(rename_all = "kebab-case")]
#[repr(C)]
pub struct Foo {
    pub field: u32,
}

fn main() {}
