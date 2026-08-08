//! `source_order` must group items by source file instead of interleaving
//! them by line number across files: everything from `alpha.rs` comes before
//! everything from `beta.rs`, even though `beta.rs` declares its items at
//! smaller line numbers than `alpha_late`/`AlphaLate`.

pub mod alpha;
pub mod beta;
