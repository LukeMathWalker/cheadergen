//! Generic `#[repr(C)]` enums (tagged unions) must be monomorphized into
//! concrete C tagged union definitions. `Either<i32>` and `Either<f32>`
//! produce separate definitions, and `MyResult<T, E>` exercises multiple
//! type parameters.

#[repr(C)]
pub enum Either<T> {
    Value(T),
    None,
}

#[repr(C)]
pub enum MyResult<T, E> {
    Ok(T),
    Err(E),
}

#[unsafe(no_mangle)]
pub extern "C" fn use_either_i32(e: Either<i32>) -> Either<i32> {
    e
}

#[unsafe(no_mangle)]
pub extern "C" fn use_either_f32(e: Either<f32>) -> Either<f32> {
    e
}

#[unsafe(no_mangle)]
pub extern "C" fn use_result(r: MyResult<i32, u8>) -> MyResult<i32, u8> {
    r
}
