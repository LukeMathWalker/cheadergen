//! Target crate: uses a generic tagged union from dep.

use partitioned_generic_enum_dep::Either;

#[unsafe(no_mangle)]
pub extern "C" fn use_either(e: Either<i32>) -> Either<i32> {
    e
}
