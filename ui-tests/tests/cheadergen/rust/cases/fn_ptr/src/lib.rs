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

// ---------------------------------------------------------------------------
// Complex types for fn-ptr testing
// ---------------------------------------------------------------------------

/// A repr(C) struct for fn-ptr testing.
#[repr(C)]
pub struct Pair {
    pub a: i32,
    pub b: i32,
}

/// A repr(C) fieldless enum for fn-ptr testing.
#[repr(C)]
pub enum Status {
    Ok,
    Error,
}

/// A repr(C) union for fn-ptr testing.
#[repr(C)]
pub union IntOrFloat {
    pub i: i32,
    pub f: f32,
}

// ---------------------------------------------------------------------------
// Function pointer typedefs with complex types
// ---------------------------------------------------------------------------

/// Fn ptr taking and returning a struct by value.
pub type StructTransform = extern "C" fn(Pair) -> Pair;

/// Fn ptr taking a struct by pointer.
pub type StructInspector = extern "C" fn(*const Pair) -> bool;

/// Fn ptr taking an enum parameter.
pub type StatusHandler = extern "C" fn(Status) -> i32;

/// Nullable fn ptr taking a union by pointer.
pub type NullableUnionCallback = Option<unsafe extern "C" fn(*mut IntOrFloat)>;

// ---------------------------------------------------------------------------
// FFI functions exercising bare fn ptrs with complex types
// ---------------------------------------------------------------------------

/// Bare fn ptr parameter: struct in, enum out.
#[unsafe(no_mangle)]
pub extern "C" fn apply_to_pair(f: extern "C" fn(Pair) -> Status) {}

/// Bare fn ptr parameter: union by value in and out.
#[unsafe(no_mangle)]
pub extern "C" fn transform_union(f: extern "C" fn(IntOrFloat) -> IntOrFloat) {}

/// Uses all four typedef'd fn ptrs together.
#[unsafe(no_mangle)]
pub extern "C" fn invoke_complex(
    transform: StructTransform,
    inspect: StructInspector,
    handle: StatusHandler,
    on_union: NullableUnionCallback,
) {}

// ---------------------------------------------------------------------------
// Struct with fn-ptr fields referencing complex types
// ---------------------------------------------------------------------------

/// Struct with fn-ptr fields that reference complex types.
#[repr(C)]
pub struct ComplexDispatcher {
    /// Non-nullable fn ptr field: struct → enum.
    pub on_pair: extern "C" fn(Pair) -> Status,
    /// Nullable fn ptr field: enum → struct pointer.
    pub on_status: Option<unsafe extern "C" fn(Status) -> *mut Pair>,
    /// Fn ptr field: union pointer → bool.
    pub on_union: extern "C" fn(*const IntOrFloat) -> bool,
}

#[unsafe(no_mangle)]
pub extern "C" fn dispatch_complex(d: ComplexDispatcher) {}
