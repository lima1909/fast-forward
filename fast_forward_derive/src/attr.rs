use quote::quote;
use syn::{parse::Parse, punctuated::Punctuated, Error, Expr, Ident, LitStr, Token};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FieldAttr {
    Store(Punctuated<Ident, Token!(::)>),
    Rename(LitStr),
}

impl Parse for FieldAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let starts_with_name_and_eq = FieldAttr::parse_name_and_eq(input);
        if !starts_with_name_and_eq {
            return match Punctuated::<Ident, Token![::]>::parse_terminated(input) {
                Ok(store) => Ok(FieldAttr::Store(store)),
                Err(err) => Err(Error::new(input.span(), format!("Invalid TypePath: {err}"))),
            };
        }

        let ident = match syn::Ident::parse(input) {
            Ok(ident) => ident,
            Err(err) => panic!("Invalid ident {err}"),
        };
        let _eq = proc_macro2::Punct::parse(input)?;

        let r = match ident.to_string().as_str() {
            "rename" => {
                let expr = Expr::parse(input)?;
                if let Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit),
                    ..
                }) = expr
                {
                    return Ok(FieldAttr::Rename(lit));
                }
                todo!()
            }
            _ => Err(Error::new_spanned(
                ident.clone(),
                format!("Invalid field attribute: {ident}"),
            )),
        };
        r
    }
}

impl FieldAttr {
    pub(crate) fn to_tokenstream(&self, field_name: Option<Ident>) -> proc_macro2::TokenStream {
        match self {
            FieldAttr::Store(ty) => quote! { #field_name: #ty, },
            FieldAttr::Rename(rename) => quote! { #rename },
        }
    }

    pub(crate) fn parse_name_and_eq(input: syn::parse::ParseStream) -> bool {
        let with_name = input.peek(Ident);
        if with_name {
            return input.peek2(Token![=]);
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attr_index() {
        let result = syn::parse_str::<FieldAttr>("fast_forward::index::uint::UIntIndex");

        let mut p = Punctuated::new();
        let span = proc_macro2::Span::call_site();
        p.push(Ident::new("fast_forward", span));
        p.push(Ident::new("index", span));
        p.push(Ident::new("uint", span));
        p.push(Ident::new("UIntIndex", span));
        assert_eq!(FieldAttr::Store(p), result.unwrap())
    }

    #[test]
    fn attr_rename() {
        let result = syn::parse_str::<FieldAttr>("rename = \"new_name\"");
        assert_eq!(
            FieldAttr::Rename(LitStr::new("new_name", proc_macro2::Span::call_site())),
            result.unwrap()
        );
    }
}
