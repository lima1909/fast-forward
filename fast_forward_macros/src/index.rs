//! ```text
//! id:    UIntIndex => 0
//! name   Store        field
//!
//! Index {
//!     name:  Ident(id)
//!     store: Type(UIntIndex),
//!     field: Ident(pk),
//! }
//! ```
//!
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Ident, Member, Result, Token, TypePath,
};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Indices(pub(crate) Vec<Index>);

impl Parse for Indices {
    fn parse(input: ParseStream) -> Result<Self> {
        let indices: Punctuated<Index, Token![,]> =
            input.parse_terminated(Index::parse, Token![,])?;

        Ok(Indices(Vec::from_iter(indices)))
    }
}

/// id:    UIntIndex => 0
/// name   Store        field
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Index {
    pub(crate) name: Ident,
    pub(crate) store: TypePath,
    pub(crate) field: Member,
}

impl Parse for Index {
    fn parse(input: ParseStream) -> Result<Self> {
        // id
        let name = input.parse::<Ident>()?;
        // :
        let _colon = input.parse::<Token![:]>()?;
        // UIntIndex
        let store = input.parse::<TypePath>()?;
        // =>
        let _arrow = input.parse::<Token![=>]>()?;
        // 0 or id
        let field = input.parse::<Member>()?;

        Ok(Index { name, store, field })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::Index as SynIndex;

    #[test]
    fn index_member_index() {
        assert_eq!(
            Index {
                name: Ident::new("id", proc_macro2::Span::call_site()),
                store: syn::parse_str::<TypePath>("UIntIndex").unwrap(),
                field: Member::Unnamed(SynIndex {
                    index: 0,
                    span: proc_macro2::Span::call_site()
                }),
            },
            syn::parse_str::<Index>("id: UIntIndex => 0").unwrap()
        );
    }

    #[test]
    fn index_member_name() {
        assert_eq!(
            Index {
                name: Ident::new("id", proc_macro2::Span::call_site()),
                store: syn::parse_str::<TypePath>("fast_forward::uint::UIntIndex").unwrap(),
                field: Member::Named(Ident::new("pk", proc_macro2::Span::call_site())),
            },
            syn::parse_str::<Index>("id: fast_forward::uint::UIntIndex => pk").unwrap()
        );
    }

    #[test]
    fn index_err_colon() {
        assert_eq!(
            "expected `:`",
            syn::parse_str::<Index>("id UIntIndex => pk")
                .unwrap_err()
                .to_string()
        );
    }

    #[test]
    fn indices() {
        let l = syn::parse_str::<Indices>("id: UIntIndex => 0, name: MapIndex => 1, ").unwrap();

        assert_eq!(2, l.0.len());
        assert_eq!(
            Indices(vec![
                Index {
                    name: Ident::new("id", proc_macro2::Span::call_site()),
                    store: syn::parse_str::<TypePath>("UIntIndex").unwrap(),
                    field: Member::Unnamed(SynIndex {
                        index: 0,
                        span: proc_macro2::Span::call_site()
                    }),
                },
                Index {
                    name: Ident::new("name", proc_macro2::Span::call_site()),
                    store: syn::parse_str::<TypePath>("MapIndex").unwrap(),
                    field: Member::Unnamed(SynIndex {
                        index: 1,
                        span: proc_macro2::Span::call_site()
                    }),
                },
            ]),
            l
        );
    }
}
