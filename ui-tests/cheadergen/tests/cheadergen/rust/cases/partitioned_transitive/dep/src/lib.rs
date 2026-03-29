//! Middle crate: defines BStruct containing CStruct from dep2.

use partitioned_transitive_dep2::CStruct;

#[repr(C)]
pub struct BStruct {
    pub c: CStruct,
    pub extra: i32,
}
