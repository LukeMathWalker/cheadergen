//! Proc-macro attributes for cheadergen.
//!
//! Provides `#[cheadergen::export]` to force inclusion of a type in the
//! generated C/C++ header, even when it is not referenced by any `extern "C"`
//! function or static.

use proc_macro::TokenStream;
use quote::quote;
use syn::{Item, parse_macro_input};

/// Force a type to be included in the generated C/C++ header.
///
/// Can be applied to structs, enums, unions, and type aliases. The type will
/// appear in the header regardless of whether it is referenced by an
/// `extern "C"` function or static.
///
/// # Example
///
/// ```ignore
/// #[cheadergen::export]
/// #[repr(C)]
/// pub struct Config {
///     pub width: u32,
///     pub height: u32,
/// }
/// ```
#[proc_macro_attribute]
pub fn export(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as Item);

    match &item {
        Item::Struct(_) | Item::Enum(_) | Item::Union(_) | Item::Type(_) => {}
        _ => {
            return syn::Error::new_spanned(
                &item,
                "#[cheadergen::export] can only be applied to structs, enums, unions, or type aliases",
            )
            .to_compile_error()
            .into();
        }
    }

    quote! {
        #[diagnostic::cheadergen::export]
        #item
    }
    .into()
}
