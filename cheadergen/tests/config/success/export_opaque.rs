#[cheadergen::config(export, opaque)]
pub struct OpaqueHandle {
    _inner: u64,
}

#[cheadergen::config(opaque)]
pub struct OpaqueHint {
    _inner: u64,
}

fn main() {}
