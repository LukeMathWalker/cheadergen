use proc_macro2::Span;
use syn::Item;
use syn::spanned::Spanned;

use crate::directive::Directive;

/// Validate directive combinations against the item kind.
/// Returns a list of errors (may be empty).
pub fn validate(directives: &[Directive], item: &Item) -> Vec<syn::Error> {
    let mut errors = Vec::new();

    let has_export = directives
        .iter()
        .any(|d| matches!(d, Directive::Export { .. }));
    let has_skip = directives
        .iter()
        .any(|d| matches!(d, Directive::Skip { .. }));

    if has_export
        && has_skip
        && let Some(skip) = directives
            .iter()
            .find(|d| matches!(d, Directive::Skip { .. }))
    {
        errors.push(syn::Error::new(
            skip.span(),
            "`export` and `skip` cannot be used on the same item",
        ));
    }

    for directive in directives {
        match directive {
            Directive::Export { .. } => match item {
                Item::Fn(f) => {
                    errors.push(syn::Error::new(
                        f.sig.fn_token.span(),
                        "`export` cannot be applied to functions",
                    ));
                }
                Item::Static(s) => {
                    errors.push(syn::Error::new(
                        s.static_token.span(),
                        "`export` cannot be applied to statics",
                    ));
                }
                _ => {}
            },
            Directive::PrefixWithName { .. } => {
                if !matches!(item, Item::Enum(_)) {
                    errors.push(syn::Error::new(
                        item_keyword_span(item),
                        "`prefix_with_name` can only be applied to enums",
                    ));
                }
            }
            Directive::FieldNames { names, span } => match item {
                Item::Struct(s) => {
                    if !matches!(s.fields, syn::Fields::Unnamed(_)) {
                        errors.push(syn::Error::new(
                            s.struct_token.span(),
                            "`field_names` can only be applied to tuple structs",
                        ));
                    } else if let syn::Fields::Unnamed(fields) = &s.fields
                        && names.len() != fields.unnamed.len()
                    {
                        errors.push(syn::Error::new(
                            *span,
                            format!(
                                "`field_names` has {} names but the struct has {} fields",
                                names.len(),
                                fields.unnamed.len()
                            ),
                        ));
                    }
                }
                _ => {
                    errors.push(syn::Error::new(
                        item_keyword_span(item),
                        "`field_names` can only be applied to tuple structs",
                    ));
                }
            },
            Directive::Skip { .. } | Directive::Rename { .. } => {}
        }
    }

    errors
}

fn item_keyword_span(item: &Item) -> Span {
    match item {
        Item::Struct(s) => s.struct_token.span(),
        Item::Enum(e) => e.enum_token.span(),
        Item::Union(u) => u.union_token.span(),
        Item::Fn(f) => f.sig.fn_token.span(),
        Item::Static(s) => s.static_token.span(),
        Item::Type(t) => t.type_token.span(),
        other => other.span(),
    }
}
