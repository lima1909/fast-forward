use quote::quote;
use syn::{parse::Parse, Error, Expr, Field, Ident, LitStr, Token, TypePath};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Attr {
    Index(TypePath),
    Name(LitStr),
}

impl Parse for Attr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        match Attr::parse_name_and_eq(input) {
            Some(ident) => match ident.to_string().as_str() {
                "name" => Attr::parse_name_attr(input),
                _ => Err(Error::new(
                    ident.span(),
                    format!("Invalid field attribute: {ident}"),
                )),
            },
            //
            // #[index(fast_forward::index::uint::UIntIndex)]
            //
            None => {
                let path = TypePath::parse(input)?;
                Ok(Attr::Index(path))
            }
        }
    }
}

impl Attr {
    fn parse_name_and_eq(input: syn::parse::ParseStream) -> Option<Ident> {
        if input.peek(Ident) && input.peek2(Token![=]) {
            let ident = Ident::parse(input).expect("expect Ident");
            let _eq = proc_macro2::Punct::parse(input);
            return Some(ident);
        }
        None
    }

    //
    // #[index(rename = "other_name")]
    //
    fn parse_name_attr(input: syn::parse::ParseStream) -> syn::Result<Self> {
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
}

#[derive(Debug, Clone)]
pub(crate) struct FieldAttrs {
    field: Field,
    index: Option<TypePath>,
    name: Option<LitStr>,
}

impl FieldAttrs {
    fn new(field: Field) -> Self {
        Self {
            field,
            index: None,
            name: None,
        }
    }

    fn add(&mut self, attr: Attr) {
        match attr {
            Attr::Index(p) => self.index = Some(p),
            Attr::Name(name) => self.name = Some(name),
        }
    }

    fn name(&self) -> Option<Ident> {
        if let Some(name) = &self.name {
            Some(Ident::new(name.value().as_str(), name.span()))
        } else {
            self.field.ident.clone()
        }
    }

    fn to_tokenstream(&self) -> Result<proc_macro2::TokenStream, Error> {
        match (self.name(), &self.index) {
            // no name and index => Err
            (None, Some(index)) => Err(Error::new_spanned(index, "Index-Field has no name")),
            // name and no index => Err
            (Some(name), None) => Err(Error::new_spanned(
                name.clone(),
                format!("Field: {name} must have an Index-Type"),
            )),
            // no name and no index => OK
            (None, None) => Ok(proc_macro2::TokenStream::new()),
            // name and index => OK
            (Some(name), Some(index)) => Ok(quote! { #name: #index, }),
        }
    }
}

pub(crate) fn from_field(field: syn::Field) -> Result<proc_macro2::TokenStream, Error> {
    let index_attrs: Vec<_> = field
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("index"))
        .collect();

    if index_attrs.is_empty() {
        return Ok(proc_macro2::TokenStream::new());
    }

    let mut field_attrs = FieldAttrs::new(field.clone());
    for attr in index_attrs {
        match attr.parse_args::<Attr>() {
            Ok(attr) => field_attrs.add(attr),
            Err(err) => {
                return Err(err);
            }
        }
    }

    field_attrs.to_tokenstream()
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
    fn name() {
        let result = syn::parse_str::<Attr>("name = \"new_name\"");
        assert_eq!(
            Attr::Name(LitStr::new("new_name", Span::call_site())),
            result.unwrap()
        );
    }

    #[test]
    fn name_with_space() {
        let result = syn::parse_str::<Attr>("name = \"new name\"");
        assert_eq!(
            Attr::Name(LitStr::new("new name", Span::call_site())),
            result.unwrap()
        );
    }

    #[test]
    fn name_no_double_quotes() {
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

    fn create_field() -> Field {
        Field {
            attrs: Vec::new(),
            vis: syn::Visibility::Inherited,
            mutability: syn::FieldMutability::None,
            ident: Some(Ident::new("pk", Span::call_site())),
            colon_token: None,
            ty: syn::Type::Path(syn::parse_str::<TypePath>("String").unwrap()),
        }
    }

    #[test]
    fn field_attrs_no_name_ok() {
        let index = syn::parse_str::<Attr>("my::Index").unwrap();

        let mut attrs = FieldAttrs::new(create_field());
        attrs.add(index);
        let token = attrs.to_tokenstream();
        assert!(token.is_ok());
    }

    #[test]
    fn field_attrs_name_and_index_ok() {
        let id = syn::parse_str::<Attr>("name = \"id\"").unwrap();
        let index = syn::parse_str::<Attr>("my::Index").unwrap();

        let mut attrs = FieldAttrs::new(create_field());
        attrs.add(id);
        attrs.add(index);
        let token = attrs.to_tokenstream();
        assert!(token.is_ok());
    }

    #[test]
    fn field_attrs_no_index_err() {
        let id = syn::parse_str::<Attr>("name = \"id\"").unwrap();

        let mut attrs = FieldAttrs::new(create_field());
        attrs.add(id);
        let token = attrs.to_tokenstream();
        assert!(token.is_err());
        assert_eq!(
            token.err().unwrap().to_string(),
            "Field: id must have an Index-Type"
        );
    }

    #[test]
    fn field_attrs_no_name_and_no_index_err() {
        let attrs = FieldAttrs::new(create_field());
        let token = attrs.to_tokenstream();
        assert!(token.is_err());
        assert_eq!(
            token.err().unwrap().to_string(),
            "Field: pk must have an Index-Type"
        );
    }
}
