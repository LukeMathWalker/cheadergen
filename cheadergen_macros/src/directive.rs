use proc_macro2::Span;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitInt, LitStr, Token};

/// Item-level directives parsed from `#[cheadergen::config(...)]`.
#[derive(Debug)]
pub enum Directive {
    Export { opaque: bool, span: Span },
    Skip { span: Span },
    Rename { name: String, span: Span },
    PrefixWithName { value: Option<bool>, span: Span },
    FieldNames { names: Vec<Ident>, span: Span },
}

impl Directive {
    pub fn span(&self) -> Span {
        match self {
            Directive::Export { span, .. }
            | Directive::Skip { span, .. }
            | Directive::Rename { span, .. }
            | Directive::PrefixWithName { span, .. }
            | Directive::FieldNames { span, .. } => *span,
        }
    }
}

/// Field/variant-level directives parsed from `#[cheadergen(...)]`.
#[derive(Debug)]
pub enum FieldDirective {
    Rename { name: String },
    Bitfield { width: u64, span: Span },
}


/// A comma-separated list of item-level directives.
pub struct Directives {
    pub items: Vec<Directive>,
}

impl Parse for Directives {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut items = Vec::new();
        while !input.is_empty() {
            items.push(input.parse()?);
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(Directives { items })
    }
}

impl Parse for Directive {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.call(Ident::parse_any)?;
        let span = ident.span();

        match ident.to_string().as_str() {
            "export" => {
                let opaque = if input.peek(syn::token::Paren) {
                    let content;
                    syn::parenthesized!(content in input);
                    let inner: Ident = content.parse()?;
                    if inner != "opaque" {
                        return Err(syn::Error::new(
                            inner.span(),
                            format!("unknown export option `{inner}`, expected `opaque`"),
                        ));
                    }
                    true
                } else {
                    false
                };
                Ok(Directive::Export { opaque, span })
            }
            "skip" => Ok(Directive::Skip { span }),
            "rename" => {
                input.parse::<Token![=]>()?;
                let lit: LitStr = input.parse()?;
                Ok(Directive::Rename {
                    name: lit.value(),
                    span,
                })
            }
            "prefix_with_name" => {
                let value = if input.peek(Token![=]) {
                    input.parse::<Token![=]>()?;
                    let lit: syn::LitBool = input.parse()?;
                    Some(lit.value)
                } else {
                    None
                };
                Ok(Directive::PrefixWithName { value, span })
            }
            "field_names" => {
                let content;
                syn::parenthesized!(content in input);
                let names = content
                    .parse_terminated(Ident::parse, Token![,])?
                    .into_iter()
                    .collect();
                Ok(Directive::FieldNames { names, span })
            }
            unknown => Err(syn::Error::new(
                span,
                format!("unknown directive `{unknown}`"),
            )),
        }
    }
}

/// A comma-separated list of field/variant-level directives.
pub struct FieldDirectives {
    pub items: Vec<FieldDirective>,
}

impl Parse for FieldDirectives {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut items = Vec::new();
        while !input.is_empty() {
            items.push(input.parse()?);
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(FieldDirectives { items })
    }
}

impl Parse for FieldDirective {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.call(Ident::parse_any)?;
        let span = ident.span();

        match ident.to_string().as_str() {
            "rename" => {
                input.parse::<Token![=]>()?;
                let lit: LitStr = input.parse()?;
                Ok(FieldDirective::Rename {
                    name: lit.value(),
                })
            }
            "bitfield" => {
                input.parse::<Token![=]>()?;
                let lit: LitInt = input.parse()?;
                Ok(FieldDirective::Bitfield {
                    width: lit.base10_parse()?,
                    span,
                })
            }
            unknown => Err(syn::Error::new(
                span,
                format!("unknown field directive `{unknown}`"),
            )),
        }
    }
}
