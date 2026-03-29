//! Target crate: uses Wrapper<MyStruct> where Wrapper and MyStruct come from different deps.

use partitioned_generic_with_dep_type_struct::MyStruct;
use partitioned_generic_with_dep_type_wrapper::Wrapper;

#[unsafe(no_mangle)]
pub extern "C" fn use_wrapped(w: Wrapper<MyStruct>) -> i32 {
    w.value.field
}
