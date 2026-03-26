//! Thread-local statics are not exported.

use std::cell::Cell;

thread_local! {
    /// A thread-local counter — should not appear in the generated header.
    pub static COUNTER: Cell<u32> = const { Cell::new(0) };
}
