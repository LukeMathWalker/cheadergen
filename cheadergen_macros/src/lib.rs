//! Proc-macro attributes for cheadergen.
//!
//! Provides `#[cheadergen::config(...)]` to control how Rust items appear in
//! the generated C/C++ header.

mod directive;
mod emit;
mod validation;

use proc_macro::TokenStream;
use quote::quote;
use syn::{Item, parse_macro_input};

use directive::Directives;

/// Control how a Rust item appears in the generated C/C++ header.
///
/// See `cheadergen::config` for the full directive reference.
#[proc_macro_attribute]
pub fn config(attr: TokenStream, item: TokenStream) -> TokenStream {
    let directives = parse_macro_input!(attr as Directives);
    let mut item = parse_macro_input!(item as Item);

    // Validate directive combinations against the item kind
    let mut errors = validation::validate(&directives.items, &item);

    // Rewrite field/variant `#[cheadergen(...)]` attrs to diagnostic form
    errors.extend(emit::rewrite_field_attrs(&mut item));

    // Emit diagnostic attributes for item-level directives
    let diagnostic_attrs = emit::emit_diagnostic_attrs(&directives.items);

    if errors.is_empty() {
        quote! {
            #(#diagnostic_attrs)*
            #item
        }
        .into()
    } else {
        // Emit errors alongside the original item for IDE recovery
        let error_tokens: Vec<_> = errors.iter().map(|e| e.to_compile_error()).collect();
        quote! {
            #(#error_tokens)*
            #(#diagnostic_attrs)*
            #item
        }
        .into()
    }
}
