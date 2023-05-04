use quote::quote;
use syn::{parse::Parse, punctuated::Punctuated, Error, Expr, Ident, LitStr, Token};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Attr {
    Index(Punctuated<Ident, Token!(::)>),
    Rename(LitStr),
}

impl Parse for Attr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        match parse_name_and_eq(input) {
            Some(ident) => {
                //
                // #[index(rename = "other_name")]
                //
                if ident.eq("rename") {
                    let expr = Expr::parse(input)?;
                    if let Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(lit),
                        ..
                    }) = expr
                    {
                        Ok(Attr::Rename(lit))
                    } else {
                        Err(Error::new(
                            input.span(),
                            r#"Expected string in double quotes: "string_value""#,
                        ))
                    }
                }
                //
                // unknown attribute ident
                //
                else {
                    Err(Error::new_spanned(
                        ident.clone(),
                        format!("Invalid field attribute: {ident}"),
                    ))
                }
            }
            None => {
                //
                // #[index(fast_forward::index::uint::UIntIndex)]
                //
                match Punctuated::<Ident, Token![::]>::parse_terminated(input) {
                    Ok(store) => Ok(Attr::Index(store)),
                    Err(err) => Err(Error::new(
                        input.span(),
                        format!("Invalid Index (TypePath): {err}"),
                    )),
                }
            }
        }
    }
}

pub(crate) fn parse_name_and_eq(input: syn::parse::ParseStream) -> Option<Ident> {
    if input.peek(Ident) && input.peek2(Token![=]) {
        let ident = Ident::parse(input).expect("expect Ident");
        let _eq = proc_macro2::Punct::parse(input);
        return Some(ident);
    }
    None
}

impl Attr {
    pub(crate) fn to_tokenstream(&self, field_name: Option<Ident>) -> proc_macro2::TokenStream {
        match self {
            Attr::Index(ty) => quote! { #field_name: #ty, },
            Attr::Rename(rename) => quote! { #rename },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::Span;

    #[test]
    fn index() {
        let result = syn::parse_str::<Attr>("fast_forward::index::uint::UIntIndex");

        let mut p = Punctuated::new();
        let span = Span::call_site();
        p.push(Ident::new("fast_forward", span));
        p.push(Ident::new("index", span));
        p.push(Ident::new("uint", span));
        p.push(Ident::new("UIntIndex", span));
        assert_eq!(Attr::Index(p), result.unwrap())
    }

    #[test]
    fn rename() {
        let result = syn::parse_str::<Attr>("rename = \"new_name\"");
        assert_eq!(
            Attr::Rename(LitStr::new("new_name", Span::call_site())),
            result.unwrap()
        );
    }

    #[test]
    fn rename_with_space() {
        let result = syn::parse_str::<Attr>("rename = \"new name\"");
        assert_eq!(
            Attr::Rename(LitStr::new("new name", Span::call_site())),
            result.unwrap()
        );
    }

    #[test]
    fn rename_no_double_quotes() {
        let result = syn::parse_str::<Attr>("rename = new_name");
        assert_eq!(
            "Expected string in double quotes: \"string_value\"",
            result.err().unwrap().to_string()
        );
    }

    #[test]
    fn lex_err() {
        let result = syn::parse_str::<Attr>("rename = \"new_name");
        assert_eq!("lex error", result.err().unwrap().to_string());
    }

    #[test]
    fn invalid_attr_name() {
        let result = syn::parse_str::<Attr>("foo = \"bar\"");
        assert_eq!(
            "Invalid field attribute: foo",
            result.err().unwrap().to_string()
        );
    }

    #[test]
    fn not_expr() {
        let result = syn::parse_str::<Attr>("rename = =");
        assert!(result
            .err()
            .unwrap()
            .to_string()
            .starts_with("unsupported expression;"));
    }
}
