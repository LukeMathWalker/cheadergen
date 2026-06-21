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
            Directive::Opaque { span } => match item {
                Item::Struct(_) | Item::Enum(_) | Item::Union(_) | Item::Type(_) => {}
                _ => {
                    errors.push(syn::Error::new(
                        *span,
                        "`opaque` can only be applied to structs, enums, unions, or type aliases",
                    ));
                }
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
            Directive::RenameAll { .. } => match item {
                Item::Struct(_) | Item::Enum(_) | Item::Union(_) => {}
                _ => {
                    errors.push(syn::Error::new(
                        item_keyword_span(item),
                        "`rename_all` can only be applied to structs, enums, or unions",
                    ));
                }
            },
            Directive::RenameAllFields { .. } => match item {
                Item::Enum(e) => {
                    let has_struct_variant = e
                        .variants
                        .iter()
                        .any(|v| matches!(v.fields, syn::Fields::Named(_)));
                    if !has_struct_variant {
                        errors.push(syn::Error::new(
                            e.enum_token.span(),
                            "`rename_all_fields` requires the enum to have at least one struct variant",
                        ));
                    }
                }
                _ => {
                    errors.push(syn::Error::new(
                        item_keyword_span(item),
                        "`rename_all_fields` can only be applied to enums",
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::directive::Directives;
    use syn::parse_quote;

    fn parse_directives(src: &str) -> Vec<Directive> {
        syn::parse_str::<Directives>(src).unwrap().items
    }

    #[test]
    fn export_plain_on_const_is_accepted() {
        let directives = parse_directives("export");
        let item: Item = parse_quote! { pub const FOO: u32 = 1; };
        assert!(validate(&directives, &item).is_empty());
    }

    #[test]
    fn opaque_on_const_is_rejected() {
        let directives = parse_directives("opaque");
        let item: Item = parse_quote! { pub const FOO: u32 = 1; };
        let errors = validate(&directives, &item);
        assert_eq!(errors.len(), 1);
        assert!(
            errors[0]
                .to_string()
                .contains("`opaque` can only be applied"),
            "unexpected error: {}",
            errors[0]
        );
    }

    #[test]
    fn export_and_opaque_on_struct_is_accepted() {
        let directives = parse_directives("export, opaque");
        let item: Item = parse_quote! { pub struct Foo { _inner: u64 } };
        assert!(validate(&directives, &item).is_empty());
    }

    #[test]
    fn opaque_on_struct_alone_is_accepted() {
        let directives = parse_directives("opaque");
        let item: Item = parse_quote! { pub struct Foo { _inner: u64 } };
        assert!(validate(&directives, &item).is_empty());
    }

    #[test]
    fn export_on_function_is_rejected() {
        let directives = parse_directives("export");
        let item: Item = parse_quote! { pub fn foo() {} };
        let errors = validate(&directives, &item);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].to_string().contains("functions"));
    }
}
