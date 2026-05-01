use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Fields, Item, Meta};

use crate::directive::{Directive, FieldDirective, FieldDirectives};

/// Emit `#[diagnostic::cheadergen::...]` attributes for each directive.
pub fn emit_diagnostic_attrs(directives: &[Directive]) -> Vec<TokenStream> {
    directives.iter().map(emit_one).collect()
}

fn emit_one(directive: &Directive) -> TokenStream {
    match directive {
        Directive::Export { opaque: false, .. } => {
            quote! { #[diagnostic::cheadergen::export] }
        }
        Directive::Export { opaque: true, .. } => {
            quote! { #[diagnostic::cheadergen::export(opaque)] }
        }
        Directive::Skip { .. } => {
            quote! { #[diagnostic::cheadergen::skip] }
        }
        Directive::Rename { name, .. } => {
            quote! { #[diagnostic::cheadergen::rename(#name)] }
        }
        Directive::PrefixWithName { value: None, .. } => {
            quote! { #[diagnostic::cheadergen::prefix_with_name] }
        }
        Directive::PrefixWithName {
            value: Some(v), ..
        } => {
            quote! { #[diagnostic::cheadergen::prefix_with_name(#v)] }
        }
        Directive::FieldNames { names, .. } => {
            quote! { #[diagnostic::cheadergen::field_names(#(#names),*)] }
        }
        Directive::RenameAll { rule, .. } => {
            let lit = rule.name();
            quote! { #[diagnostic::cheadergen::rename_all(#lit)] }
        }
        Directive::RenameAllFields { rule, .. } => {
            let lit = rule.name();
            quote! { #[diagnostic::cheadergen::rename_all_fields(#lit)] }
        }
    }
}

fn emit_field_diagnostic(directive: &FieldDirective) -> TokenStream {
    match directive {
        FieldDirective::Rename { name, .. } => {
            quote! { #[diagnostic::cheadergen::rename(#name)] }
        }
        FieldDirective::Bitfield { width, .. } => {
            quote! { #[diagnostic::cheadergen::bitfield(#width)] }
        }
    }
}

/// Rewrite `#[cheadergen(...)]` attributes on fields and variants to
/// `#[diagnostic::cheadergen::...]` attributes.
///
/// Returns any parse/validation errors encountered.
pub fn rewrite_field_attrs(item: &mut Item) -> Vec<syn::Error> {
    let mut errors = Vec::new();

    match item {
        Item::Struct(s) => {
            rewrite_fields(&mut s.fields, false, &mut errors);
        }
        Item::Enum(e) => {
            for variant in &mut e.variants {
                rewrite_attrs_on_list(&mut variant.attrs, true, &mut errors);
                rewrite_fields(&mut variant.fields, false, &mut errors);
            }
        }
        Item::Union(u) => {
            for field in &mut u.fields.named {
                rewrite_attrs_on_list(&mut field.attrs, false, &mut errors);
            }
        }
        _ => {}
    }

    errors
}

fn rewrite_fields(fields: &mut Fields, is_variant: bool, errors: &mut Vec<syn::Error>) {
    match fields {
        Fields::Named(named) => {
            for field in &mut named.named {
                rewrite_attrs_on_list(&mut field.attrs, is_variant, errors);
            }
        }
        Fields::Unnamed(unnamed) => {
            for field in &mut unnamed.unnamed {
                rewrite_attrs_on_list(&mut field.attrs, is_variant, errors);
            }
        }
        Fields::Unit => {}
    }
}

fn rewrite_attrs_on_list(
    attrs: &mut Vec<Attribute>,
    is_variant: bool,
    errors: &mut Vec<syn::Error>,
) {
    let mut new_attrs = Vec::new();
    let mut i = 0;
    while i < attrs.len() {
        if is_cheadergen_attr(&attrs[i]) {
            let attr = attrs.remove(i);
            match parse_and_rewrite(attr, is_variant) {
                Ok(replacement_tokens) => {
                    new_attrs.push((i, replacement_tokens));
                }
                Err(e) => errors.push(e),
            }
        } else {
            i += 1;
        }
    }
    // Insert replacement diagnostic attributes at original positions
    for (idx, tokens) in new_attrs.into_iter().rev() {
        let replacement_attrs: Vec<Attribute> =
            syn::parse2::<AttrsWrapper>(tokens).map_or_else(|_| vec![], |w| w.0);
        for (j, attr) in replacement_attrs.into_iter().enumerate() {
            attrs.insert(idx + j, attr);
        }
    }
}

/// Helper to parse a token stream as a sequence of outer attributes.
struct AttrsWrapper(Vec<Attribute>);

impl syn::parse::Parse for AttrsWrapper {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(AttrsWrapper(Attribute::parse_outer(input)?))
    }
}

fn is_cheadergen_attr(attr: &Attribute) -> bool {
    // Match `#[cheadergen(...)]` — single path segment "cheadergen"
    let path = attr.path();
    if path.segments.len() != 1 {
        return false;
    }
    path.segments[0].ident == "cheadergen"
}

fn parse_and_rewrite(
    attr: Attribute,
    is_variant: bool,
) -> Result<TokenStream, syn::Error> {
    let directives: FieldDirectives = match &attr.meta {
        Meta::List(list) => syn::parse2(list.tokens.clone())?,
        _ => {
            return Err(syn::Error::new_spanned(
                &attr,
                "expected `#[cheadergen(...)]`",
            ));
        }
    };

    let mut tokens = TokenStream::new();
    for directive in &directives.items {
        // Validate: bitfield not allowed on variants
        if is_variant
            && let FieldDirective::Bitfield { span, .. } = directive
        {
            return Err(syn::Error::new(
                *span,
                "`bitfield` cannot be applied to enum variants",
            ));
        }
        tokens.extend(emit_field_diagnostic(directive));
    }
    Ok(tokens)
}
