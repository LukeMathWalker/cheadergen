//! Checks that angle-bracketed entries in `includes` are emitted as
//! system includes (`#include <time.h>`) rather than being double-quoted
//! into `#include "<time.h>"`.

#[unsafe(no_mangle)]
pub extern "C" fn timestamp() -> u64 {
    0
}
