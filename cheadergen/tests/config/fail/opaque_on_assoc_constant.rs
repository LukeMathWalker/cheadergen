pub struct Foo;

impl Foo {
    #[cheadergen::config(opaque)]
    pub const BAR: u32 = 1;
}

fn main() {}
