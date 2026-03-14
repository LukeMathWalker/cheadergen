//! Associated constants on exported types are emitted as `#define TypeName_CONST value`.
//! Only `pub` constants are exported; `pub(crate)` and private are skipped.
//! Computed associated constants (const fn) are skipped — rustdoc uses `"_"` as a placeholder.

#[repr(C)]
pub struct Foo {}

const fn compute() -> i32 {
    99
}

impl Foo {
    // Integer types
    pub const U8: u8 = 1;
    pub const U16: u16 = 2;
    pub const U32: u32 = 3;
    pub const U64: u64 = 4;
    pub const USIZE: usize = 5;
    pub const I8: i8 = -1;
    pub const I16: i16 = -2;
    pub const I32: i32 = -3;
    pub const I64: i64 = -4;
    pub const ISIZE: isize = -5;

    // Float types
    pub const F32: f32 = 3.14;
    pub const F64: f64 = 2.718;

    // Bool
    pub const YES: bool = true;
    pub const NO: bool = false;

    // Char
    pub const CH: char = 'A';

    // &str
    pub const GREETING: &'static str = "hello world";

    // Computed value — skipped (rustdoc's AssocConst uses "_" placeholder for const fn results,
    // unlike free-standing Constant which evaluates them)
    pub const COMPUTED: i32 = compute();

    // Visibility filters
    pub(crate) const DONT_EXPORT_CRATE: i32 = 20;
    const DONT_EXPORT_PRIV: i32 = 30;
}

#[unsafe(no_mangle)]
pub extern "C" fn root(x: Foo) {}
