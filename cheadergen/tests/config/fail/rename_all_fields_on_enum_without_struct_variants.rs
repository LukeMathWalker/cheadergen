#[cheadergen::config(rename_all_fields = "camelCase")]
#[repr(C)]
pub enum FieldlessOnly {
    A,
    B,
    C,
}

#[cheadergen::config(rename_all_fields = "camelCase")]
#[repr(C)]
pub enum TupleOnly {
    Tup(u32),
    Plain,
}

fn main() {}
