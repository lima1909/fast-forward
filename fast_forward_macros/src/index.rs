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
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Ident, Member, Result, Token, TypePath,
};

use crate::list::Type;

///
/// List of indices
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Indices(pub(crate) Vec<Index>);

impl Parse for Indices {
    fn parse(input: ParseStream) -> Result<Self> {
        let indices: Punctuated<Index, Token![,]> =
            input.parse_terminated(Index::parse, Token![,])?;

        Ok(Indices(Vec::from_iter(indices)))
    }
}

impl Indices {
    pub(crate) fn to_declare_struct_field_tokens(&self) -> impl Iterator<Item = TokenStream> + '_ {
        self.0.iter().map(|i| i.to_declare_struct_field_tokens())
    }

    pub(crate) fn to_init_struct_field_tokens(&self, on: &TypePath) -> Vec<TokenStream> {
        self.0
            .iter()
            .map(|i| i.to_init_struct_field_tokens(on))
            .collect::<Vec<_>>()
    }

    pub(crate) fn to_retrieve_tokens<'a>(
        &'a self,
        typ: &'a Type,
        on: &'a TypePath,
    ) -> impl Iterator<Item = TokenStream> + 'a {
        self.0.iter().map(|i| i.to_retrieve_tokens(typ, on))
    }
}

///
/// id:    UIntIndex => 0[.clone]
/// name   store        field[.method]
///
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Index {
    pub(crate) name: Ident,
    pub(crate) store: TypePath,
    pub(crate) field: Member,
    pub(crate) method: Option<Ident>,
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

        // optional point with method
        let mut method = None;
        if input.peek(Token![.]) {
            let _p = input.parse::<Token![.]>();
            method = Some(input.parse::<Ident>()?);
        }

        Ok(Index {
            name,
            store,
            field,
            method,
        })
    }
}

impl Index {
    pub(crate) fn to_declare_struct_field_tokens(&self) -> TokenStream {
        let name = self.name.clone();
        let store = self.store.clone();

        // ids: UIntIndex,
        quote! { #name: #store, }
    }

    pub(crate) fn to_init_struct_field_tokens(&self, on: &TypePath) -> TokenStream {
        let name = self.name.clone();
        let field = self.field.clone();
        let method = self.method.clone();

        if let Some(method) = method {
            quote! {
                #name: items.to_store(|o: &#on| o.#field.#method()),
            }
        } else {
            quote! {
                #name: items.to_store(|o: &#on| o.#field),
            }
        }
    }

    pub(crate) fn to_retrieve_tokens(&self, typ: &Type, on: &TypePath) -> TokenStream {
        let name = self.name.clone();
        let store = self.store.clone();

        match typ {
            Type::List => {
                quote! {
                    pub fn #name(&self) -> fast_forward::collections::Retriever<'_, #store, L> {
                        fast_forward::collections::Retriever::new(&self.#name, &self.items)
                    }
                }
            }
            Type::RefList => {
                quote! {
                    pub fn #name(&self) -> fast_forward::collections::Retriever<'_, #store, fast_forward::collections::ro::Slice<'a, #on>> {
                        fast_forward::collections::Retriever::new(&self.#name, &self.items)
                    }
                }
            }
            Type::Map => {
                quote! {
                    pub fn #name(&self) -> fast_forward::collections::Retriever<'_, #store, M> {
                        fast_forward::collections::Retriever::new(&self.#name, &self.items)
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote, Index as SynIndex};

    #[test]
    fn to_field_declare_tokens() {
        let idx = syn::parse_str::<Index>("id: UIntIndex => 0").unwrap();

        let ts = idx.to_declare_struct_field_tokens();
        let ts2: TokenStream = parse_quote!(id: UIntIndex,);

        assert_eq!(ts.to_string(), ts2.to_string());
    }

    #[test]
    fn to_init_struct_field_tokens() {
        let idx = syn::parse_str::<Index>("id: UIntIndex => 0").unwrap();
        let on = syn::parse_str::<TypePath>("Car").unwrap();

        let ts = idx.to_init_struct_field_tokens(&on);
        let ts2: TokenStream = parse_quote!(id: items.to_store(|o: &Car| o.0),);

        assert_eq!(ts.to_string(), ts2.to_string());
    }

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
                method: None,
            },
            syn::parse_str::<Index>("id: UIntIndex => 0").unwrap()
        );
    }

    #[test]
    fn index_member_index_method() {
        assert_eq!(
            Index {
                name: Ident::new("name", proc_macro2::Span::call_site()),
                store: syn::parse_str::<TypePath>("MapIndex").unwrap(),
                field: Member::Unnamed(SynIndex {
                    index: 0,
                    span: proc_macro2::Span::call_site()
                }),
                method: Some(Ident::new("clone", proc_macro2::Span::call_site())),
            },
            syn::parse_str::<Index>("name: MapIndex => 0.clone").unwrap()
        );
    }

    #[test]
    fn index_member_name() {
        assert_eq!(
            Index {
                name: Ident::new("id", proc_macro2::Span::call_site()),
                store: syn::parse_str::<TypePath>("fast_forward::uint::UIntIndex").unwrap(),
                field: Member::Named(Ident::new("pk", proc_macro2::Span::call_site())),
                method: None,
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
                    method: None,
                },
                Index {
                    name: Ident::new("name", proc_macro2::Span::call_site()),
                    store: syn::parse_str::<TypePath>("MapIndex").unwrap(),
                    field: Member::Unnamed(SynIndex {
                        index: 1,
                        span: proc_macro2::Span::call_site()
                    }),
                    method: None,
                },
            ]),
            l
        );
    }
}
