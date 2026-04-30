//! Configure how `cheadergen` generates C/C++ headers from Rust.
//!
//! `cheadergen` generates C/C++ header files from Rust libraries that expose a
//! `pub extern "C"` API. Output is controlled in two complementary ways:
//!
//! - **[`cheadergen.toml`](config_reference)** — a project-wide TOML file that
//!   the CLI loads via `--config`. It sets the preamble, include guards, sort
//!   order, per-package opaque/skip rules, per-header overrides, and
//!   language-specific options. See the [config reference](config_reference)
//!   for the full schema.
//! - **Per-item attributes** — `#[cheadergen::config(...)]` on items and
//!   `#[cheadergen(...)]` on fields and variants, placed in your Rust source
//!   to control how an individual definition appears in the generated header.
//!   See the [`config`] macro for the directive reference.
//!
//! # Per-item attributes
//!
//! There are two attribute levels:
//!
//! - [`#[cheadergen::config(...)]`](config) — placed on items (structs, enums,
//!   unions, functions, statics, type aliases) to control their header
//!   representation.
//! - `#[cheadergen(...)]` — placed on fields and enum variants to control
//!   their individual representation within a parent item.
//!
//! ## Example
//!
//! ```ignore
//! #[cheadergen::config(export, rename = "CColor")]
//! #[repr(C)]
//! pub enum Color {
//!     #[cheadergen(rename = "COLOR_RED")]
//!     Red,
//!     Green,
//!     Blue,
//! }
//!
//! #[cheadergen::config(export, field_names(x, y))]
//! #[repr(C)]
//! pub struct Point2D(pub f64, pub f64);
//!
//! #[cheadergen::config(skip)]
//! #[unsafe(no_mangle)]
//! pub extern "C" fn internal_helper() {}
//! ```

/// Control how a Rust item appears in the generated C/C++ header.
///
/// # Example
///
/// ```ignore
/// #[cheadergen::config(export, rename = "CConfig")]
/// //                   ~~~~~~  ~~~~~~~~~~~~~~~~~~
/// //                   │       └ Override the C name
/// //                   └ Force inclusion in the header
/// #[repr(C)]
/// pub struct Config {
///     #[cheadergen(rename = "raw_width")]
///     //           ~~~~~~~~~~~~~~~~~~~~
///     //           └ Override the C field name
///     pub width: u32,
///     #[cheadergen(bitfield = 8)]
///     //           ~~~~~~~~~~~
///     //           └ Emit as a C bitfield with width 8
///     pub flags: u8,
/// }
/// ```
///
/// # Directives
///
/// | Name | Applies to | Required |
/// |------|-----------|----------|
/// | [`export`](#export) | struct, enum, union, type alias | No |
/// | [`skip`](#skip) | struct, enum, union, type alias, function, static | No |
/// | [`rename`](#rename) | struct, enum, union, type alias, function, static | No |
/// | [`prefix_with_name`](#prefix_with_name) | enum | No |
/// | [`field_names`](#field_names) | tuple struct | No |
///
/// ## `export`
///
/// Force the item to be included in the generated header. By default, `cheadergen` only includes
/// items that are reachable from `pub extern "C"` functions. Use `export` to include items that
/// are not directly reachable but should still appear in the header.
///
/// ```ignore
/// #[cheadergen::config(export)]
/// //                   ~~~~~~
/// //                   └ Include this struct in the header
/// #[repr(C)]
/// pub struct Config {
///     pub width: u32,
/// }
/// ```
///
/// Pass `opaque` to emit the type as a forward declaration without exposing its layout:
///
/// ```ignore
/// #[cheadergen::config(export(opaque))]
/// //                   ~~~~~~~~~~~~~~
/// //                   └ Include as an opaque (forward-declared) type
/// pub struct OpaqueHandle {
///     _inner: u64,
/// }
/// ```
///
/// Cannot be applied to functions or statics (they are included via `#[unsafe(no_mangle)]`).
/// Cannot be combined with [`skip`](#skip).
///
/// ## `skip`
///
/// Exclude the item from the generated header, even if it would otherwise be included.
///
/// ```ignore
/// #[cheadergen::config(skip)]
/// //                   ~~~~
/// //                   └ Exclude this function from the header
/// #[unsafe(no_mangle)]
/// pub extern "C" fn internal_helper() {}
/// ```
///
/// Cannot be combined with [`export`](#export).
///
/// ## `rename`
///
/// Override the C name of the item. By default, the Rust identifier is used as-is.
///
/// ```ignore
/// #[cheadergen::config(rename = "CPoint")]
/// //                   ~~~~~~~~~~~~~~~~~
/// //                   └ Use "CPoint" instead of "Point" in the header
/// #[repr(C)]
/// pub struct Point {
///     pub x: f64,
///     pub y: f64,
/// }
/// ```
///
/// ## `prefix_with_name`
///
/// Prefix each variant of a C enum with the enum's name, separated by an underscore.
/// This is common in C to avoid name collisions since C enums share a global namespace.
///
/// ```ignore
/// #[cheadergen::config(prefix_with_name)]
/// //                   ~~~~~~~~~~~~~~~~
/// //                   └ Variants become Status_Ok, Status_Error, ...
/// #[repr(C)]
/// pub enum Status {
///     Ok,
///     Error,
/// }
/// ```
///
/// Use `prefix_with_name = false` to explicitly disable the behavior (useful when overriding
/// a default configuration).
///
/// Can only be applied to enums.
///
/// ## `field_names`
///
/// Assign C field names to the fields of a tuple struct. The number of names must match
/// the number of fields.
///
/// ```ignore
/// #[cheadergen::config(field_names(x, y))]
/// //                   ~~~~~~~~~~~~~~~~~
/// //                   └ Name the two fields "x" and "y" in the C header
/// #[repr(C)]
/// pub struct Point2D(pub f64, pub f64);
/// ```
///
/// Can only be applied to tuple structs.
///
/// # Field and variant directives
///
/// Fields and enum variants use `#[cheadergen(...)]` (without `::config`).
///
/// | Name | Applies to | Required |
/// |------|-----------|----------|
/// | [`rename`](#field-rename) | field, variant | No |
/// | [`bitfield`](#bitfield) | field (not variant) | No |
///
/// ## `rename` (field) {#field-rename}
///
/// Override the C name of a struct field or enum variant.
///
/// ```ignore
/// #[cheadergen::config(export)]
/// #[repr(C)]
/// pub enum Color {
///     #[cheadergen(rename = "COLOR_RED")]
///     //           ~~~~~~~~~~~~~~~~~~~~
///     //           └ Use "COLOR_RED" instead of "Red"
///     Red,
///     Green,
/// }
/// ```
///
/// ## `bitfield`
///
/// Emit the field as a C bitfield with the given width in bits.
///
/// ```ignore
/// #[cheadergen::config(export)]
/// #[repr(C)]
/// pub struct Flags {
///     #[cheadergen(bitfield = 4)]
///     //           ~~~~~~~~~~~
///     //           └ Emit as a 4-bit bitfield
///     pub mode: u8,
/// }
/// ```
///
/// Cannot be applied to enum variants.
pub use cheadergen_macros::config;

pub mod config_reference;
