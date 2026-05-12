use proc_macro2::Span;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitInt, LitStr, Token};

/// Casing rule used by `rename_all` and `rename_all_fields`.
///
/// The accepted literals (case-sensitive) match serde:
/// `"camelCase"`, `"PascalCase"`, `"snake_case"`, `"SCREAMING_SNAKE_CASE"`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[expect(
    clippy::enum_variant_names,
    reason = "the variant names mirror the canonical serde casing literals"
)]
pub enum RenameRule {
    PascalCase,
    CamelCase,
    SnakeCase,
    ScreamingSnakeCase,
}

impl RenameRule {
    /// Parse the literal accepted in `rename_all = "..."`.
    pub fn from_literal(s: &str) -> Result<Self, String> {
        match s {
            "PascalCase" => Ok(Self::PascalCase),
            "camelCase" => Ok(Self::CamelCase),
            "snake_case" => Ok(Self::SnakeCase),
            "SCREAMING_SNAKE_CASE" => Ok(Self::ScreamingSnakeCase),
            other => Err(format!(
                "unknown casing rule `{other}`; expected one of `camelCase`, `PascalCase`, `snake_case`, `SCREAMING_SNAKE_CASE`"
            )),
        }
    }

    /// The canonical literal name (round-trips through diagnostic emission).
    pub fn name(&self) -> &'static str {
        match self {
            Self::PascalCase => "PascalCase",
            Self::CamelCase => "camelCase",
            Self::SnakeCase => "snake_case",
            Self::ScreamingSnakeCase => "SCREAMING_SNAKE_CASE",
        }
    }
}

/// Item-level directives parsed from `#[cheadergen::config(...)]`.
#[derive(Debug)]
pub enum Directive {
    Export { opaque: bool, span: Span },
    Skip { span: Span },
    Rename { name: String, span: Span },
    PrefixWithName { value: Option<bool>, span: Span },
    FieldNames { names: Vec<Ident>, span: Span },
    RenameAll { rule: RenameRule, span: Span },
    RenameAllFields { rule: RenameRule, span: Span },
}

impl Directive {
    pub fn span(&self) -> Span {
        match self {
            Directive::Export { span, .. }
            | Directive::Skip { span, .. }
            | Directive::Rename { span, .. }
            | Directive::PrefixWithName { span, .. }
            | Directive::FieldNames { span, .. }
            | Directive::RenameAll { span, .. }
            | Directive::RenameAllFields { span, .. } => *span,
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
                let name = lit.value();
                if name.is_empty() {
                    return Err(syn::Error::new(lit.span(), "`rename` cannot be empty"));
                }
                Ok(Directive::Rename { name, span })
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
            "rename_all" => {
                input.parse::<Token![=]>()?;
                let lit: LitStr = input.parse()?;
                let rule = RenameRule::from_literal(&lit.value())
                    .map_err(|msg| syn::Error::new(lit.span(), msg))?;
                Ok(Directive::RenameAll { rule, span })
            }
            "rename_all_fields" => {
                input.parse::<Token![=]>()?;
                let lit: LitStr = input.parse()?;
                let rule = RenameRule::from_literal(&lit.value())
                    .map_err(|msg| syn::Error::new(lit.span(), msg))?;
                Ok(Directive::RenameAllFields { rule, span })
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
                let name = lit.value();
                if name.is_empty() {
                    return Err(syn::Error::new(lit.span(), "`rename` cannot be empty"));
                }
                Ok(FieldDirective::Rename { name })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_literal_rejects_unknown() {
        assert!(RenameRule::from_literal("kebab-case").is_err());
        assert!(RenameRule::from_literal("camelcase").is_err());
        assert!(RenameRule::from_literal("").is_err());
    }

    #[test]
    fn from_literal_accepts_canonical() {
        assert!(matches!(
            RenameRule::from_literal("camelCase"),
            Ok(RenameRule::CamelCase)
        ));
        assert!(matches!(
            RenameRule::from_literal("PascalCase"),
            Ok(RenameRule::PascalCase)
        ));
        assert!(matches!(
            RenameRule::from_literal("snake_case"),
            Ok(RenameRule::SnakeCase)
        ));
        assert!(matches!(
            RenameRule::from_literal("SCREAMING_SNAKE_CASE"),
            Ok(RenameRule::ScreamingSnakeCase)
        ));
    }

    #[test]
    fn name_round_trips() {
        for r in [
            RenameRule::PascalCase,
            RenameRule::CamelCase,
            RenameRule::SnakeCase,
            RenameRule::ScreamingSnakeCase,
        ] {
            assert_eq!(RenameRule::from_literal(r.name()).unwrap(), r);
        }
    }
}
