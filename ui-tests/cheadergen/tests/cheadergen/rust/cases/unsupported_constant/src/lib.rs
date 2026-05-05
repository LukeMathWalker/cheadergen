//! Constants whose types cannot be represented in C (e.g. `u128`, `i128`)
//! produce a generation error when the user explicitly opts in via
//! `#[cheadergen::config(export)]`.

#[cheadergen::config(export)]
pub const BIG_UNSIGNED: u128 = 340_282_366_920_938_463_463_374_607_431_768_211_455;

#[cheadergen::config(export)]
pub const BIG_SIGNED: i128 = -170_141_183_460_469_231_731_687_303_715_884_105_728;

#[unsafe(no_mangle)]
pub extern "C" fn anchor() {}
