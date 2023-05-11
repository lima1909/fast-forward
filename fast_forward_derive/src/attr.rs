use quote::quote;
use syn::{parse::Parse, Error, Expr, Field, Ident, LitStr, Token, TypePath};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Attr {
    Index(TypePath),
    Name(LitStr),
}

impl Parse for Attr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        match parse_name_and_eq(input) {
            Some(ident) => {
                //
                // #[index(rename = "other_name")]
                //
                if ident.eq("name") {
                    let expr = Expr::parse(input)?;
                    if let Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(lit),
                        ..
                    }) = expr
                    {
                        Ok(Attr::Name(lit))
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
                match TypePath::parse(input) {
                    Ok(path) => Ok(Attr::Index(path)),
                    Err(err) => Err(Error::new(
                        input.span(),
                        format!("Invalid Index format: {err}"),
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

#[derive(Debug, Clone)]
pub(crate) struct FieldAttrs {
    field: syn::Field,
    index: Option<TypePath>,
    name: Option<LitStr>,
}

impl FieldAttrs {
    pub(crate) fn new(field: syn::Field) -> Self {
        Self {
            field,
            index: None,
            name: None,
        }
    }

    pub(crate) fn add(&mut self, attr: Attr) {
        match attr {
            Attr::Index(p) => self.index = Some(p),
            Attr::Name(name) => self.name = Some(name),
        }
    }

    pub(crate) fn name(&self) -> syn::Result<Ident> {
        if let Some(name) = &self.name {
            Ok(Ident::new(name.value().as_str(), name.span()))
        } else {
            match &self.field.ident {
                Some(ident) => Ok(ident.clone()),
                None => Err(Error::new_spanned(
                    self.field.clone(),
                    "Could not create a Index for an unnamed field",
                )),
            }
        }
    }

    pub(crate) fn index(&self) -> syn::Result<TypePath> {
        if let Some(path) = &self.index {
            Ok(path.clone())
        } else {
            Err(Error::new_spanned(
                self.field.clone(),
                "Index is a mandatory field",
            ))
        }
    }

    pub(crate) fn to_tokenstream(&self) -> proc_macro2::TokenStream {
        let field_name = self.name().unwrap();
        let ty = self.index().unwrap();

        quote! { #field_name: #ty, }
    }
}

impl TryFrom<Field> for FieldAttrs {
    type Error = &'static str;

    fn try_from(field: Field) -> Result<Self, Self::Error> {
        if field.attrs.is_empty() {
            return Err("");
        }

        Err("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::Span;

    #[test]
    fn index() {
        let result = syn::parse_str::<Attr>("fast_forward::index::uint::UIntIndex").unwrap();
        let path = syn::parse_str::<TypePath>("fast_forward::index::uint::UIntIndex").unwrap();
        assert_eq!(Attr::Index(path), result)
    }

    #[test]
    fn rename() {
        let result = syn::parse_str::<Attr>("name = \"new_name\"");
        assert_eq!(
            Attr::Name(LitStr::new("new_name", Span::call_site())),
            result.unwrap()
        );
    }

    #[test]
    fn rename_with_space() {
        let result = syn::parse_str::<Attr>("name = \"new name\"");
        assert_eq!(
            Attr::Name(LitStr::new("new name", Span::call_site())),
            result.unwrap()
        );
    }

    #[test]
    fn rename_no_double_quotes() {
        let result = syn::parse_str::<Attr>("name = new_name");
        assert_eq!(
            "Expected string in double quotes: \"string_value\"",
            result.err().unwrap().to_string()
        );
    }

    #[test]
    fn lex_err() {
        let result = syn::parse_str::<Attr>("name = \"new_name");
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
        let result = syn::parse_str::<Attr>("name = =");
        assert!(result
            .err()
            .unwrap()
            .to_string()
            .starts_with("unsupported expression;"));
    }
}
