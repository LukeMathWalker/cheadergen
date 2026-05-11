//! `#[repr(C, align(N))]` on an enum is rejected: alignment support for
//! enums is out of scope for the initial implementation.

#[repr(C, align(8))]
pub enum AlignedEnum {
    A,
    B,
    C,
}

#[unsafe(no_mangle)]
pub extern "C" fn aligned_enum_value() -> AlignedEnum {
    AlignedEnum::A
}
