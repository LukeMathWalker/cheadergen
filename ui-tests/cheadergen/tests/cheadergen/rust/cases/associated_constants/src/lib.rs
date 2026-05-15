//! Associated constants on exported types are emitted as `#define TypeName_CONST value`.
//!
//! Each assoc constant is opt-in: it must carry `#[cheadergen::config(export)]`
//! to reach the header — exporting the parent type does not cascade.

#[repr(C)]
pub struct Foo {}

const fn compute() -> i32 {
    99
}

impl Foo {
    // Integer types
    #[cheadergen::config(export)]
    pub const U8: u8 = 1;
    #[cheadergen::config(export)]
    pub const U16: u16 = 2;
    #[cheadergen::config(export)]
    pub const U32: u32 = 3;
    #[cheadergen::config(export)]
    pub const U64: u64 = 4;
    #[cheadergen::config(export)]
    pub const USIZE: usize = 5;
    #[cheadergen::config(export)]
    pub const I8: i8 = -1;
    #[cheadergen::config(export)]
    pub const I16: i16 = -2;
    #[cheadergen::config(export)]
    pub const I32: i32 = -3;
    #[cheadergen::config(export)]
    pub const I64: i64 = -4;
    #[cheadergen::config(export)]
    pub const ISIZE: isize = -5;

    // Float types
    #[cheadergen::config(export)]
    pub const F32: f32 = 3.14;
    #[cheadergen::config(export)]
    pub const F64: f64 = 2.718;

    // Bool
    #[cheadergen::config(export)]
    pub const YES: bool = true;
    #[cheadergen::config(export)]
    pub const NO: bool = false;

    // Char
    #[cheadergen::config(export)]
    pub const CH: char = 'A';

    // &str
    #[cheadergen::config(export)]
    pub const GREETING: &'static str = "hello world";

    // Computed value — un-annotated, so silently excluded (annotating it
    // would be a hard error since rustdoc cannot evaluate const fn results).
    pub const COMPUTED: i32 = compute();

    // Non-public — would also need an annotation, but rustdoc doesn't
    // surface them either way.
    pub(crate) const DONT_EXPORT_CRATE: i32 = 20;
    const DONT_EXPORT_PRIV: i32 = 30;
}

#[unsafe(no_mangle)]
#[expect(improper_ctypes_definitions)]
pub extern "C" fn root(x: Foo) {}
