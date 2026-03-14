//! Function pointer support: bare fn ptrs, fn ptr typedefs, fn ptrs in struct fields,
//! and Option<fn> → fn simplification.

/// A callback that takes two integers and returns a boolean.
pub type Predicate = extern "C" fn(i32, i32) -> bool;

/// A nullable callback (Option<fn> should simplify to bare fn ptr).
pub type NullableCallback = Option<unsafe extern "C" fn(u64)>;

#[repr(C)]
pub struct Dispatcher {
    /// A function pointer field.
    pub on_event: extern "C" fn(u32),
    /// A nullable function pointer field.
    pub on_error: Option<unsafe extern "C" fn(i32)>,
}

/// Takes a bare function pointer as a parameter.
#[unsafe(no_mangle)]
pub extern "C" fn register_callback(cb: extern "C" fn(u8) -> bool) {}

/// Takes typedef'd function pointer types.
#[unsafe(no_mangle)]
pub extern "C" fn invoke(pred: Predicate, fallback: NullableCallback) {}

/// Uses a struct with function pointer fields.
#[unsafe(no_mangle)]
pub extern "C" fn dispatch(d: Dispatcher) {}
