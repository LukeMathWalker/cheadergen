//! Co-recursive types: `Parent` contains `Child` by value, while `Child`
//! contains a pointer back to `Parent`. Similarly, `Ping` and `Pong` are
//! co-recursive enums. The generated header must emit full definitions
//! for all types, not opaque forward declarations.

#[repr(C)]
pub struct Child {
    pub value: u8,
    pub parent: *const Parent,
}

#[repr(C)]
pub struct Parent {
    pub id: i32,
    pub child: Child,
}

#[repr(C)]
pub enum Pong {
    Done,
    Continue {
        value: u8,
        back: *const Ping,
    },
}

#[repr(C)]
pub enum Ping {
    Done,
    Continue {
        value: i32,
        next: Pong,
    },
}

#[unsafe(no_mangle)]
pub extern "C" fn use_parent(p: Parent) -> Parent {
    p
}

#[unsafe(no_mangle)]
pub extern "C" fn use_ping(p: Ping) -> Ping {
    p
}
