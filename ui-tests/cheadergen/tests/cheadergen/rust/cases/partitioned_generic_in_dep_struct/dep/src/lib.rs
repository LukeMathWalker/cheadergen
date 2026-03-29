//! Middle crate: defines BStruct containing Wrapper<i32> from dep2.

use partitioned_generic_in_dep_struct_dep2::Wrapper;

#[repr(C)]
pub struct BStruct {
    pub wrapped: Wrapper<i32>,
    pub extra: i32,
}
