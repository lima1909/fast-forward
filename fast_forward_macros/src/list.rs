//! ```text
//! create [ro | rw | rwd] Cars on Car
//! kw     Kind            name kw on(type)
//!
//! List {
//!     name: Ident(Cars)
//!     kind: Kind::RO,
//!     on: Type(Car),
//! }
//! ```
//!
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    Ident, Result, TypePath,
};

use crate::index::Indices;

mod keyword {
    use syn::custom_keyword;

    custom_keyword!(create);
    custom_keyword!(on);
    custom_keyword!(using);

    // Kinds
    custom_keyword!(ro);
    custom_keyword!(rw);
    custom_keyword!(rwd);
}

/// create [ro | rw | rwd] Cars on Car
/// kw     Kind            name kw on(type)
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct IndexedList {
    pub(crate) name: Ident,
    pub(crate) kind: Kind,
    pub(crate) on: TypePath,
    pub(crate) indices: Indices,
}

impl Parse for IndexedList {
    fn parse(input: ParseStream) -> Result<Self> {
        // create
        let _kw_create = input.parse::<keyword::create>()?;
        // [ro | rw | rwd]
        let kind = input.parse::<Kind>()?;
        // Cars
        let name = input.parse::<Ident>()?;
        // on
        let _kw_on = input.parse::<keyword::on>()?;
        // Car
        let on = input.parse::<TypePath>()?;
        // using
        let _kw_using = input.parse::<keyword::using>()?;

        // { id: UIntIndex => 0 }
        let index_list;
        let _brace = braced!(index_list in input);
        let indices = index_list.parse::<Indices>()?;

        Ok(Self {
            name,
            kind,
            on,
            indices,
        })
    }
}

impl ToTokens for IndexedList {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let list_name = self.name.clone();
        let on = self.on.clone();

        tokens.extend(self.create_struct(&list_name, &on));
        tokens.extend(self.impl_new(&list_name, &on));
        tokens.extend(self.retrieve(&list_name, &on));
        tokens.extend(self.impl_deref(&list_name, &on));
        tokens.extend(self.impl_index(&list_name, &on));
    }
}

impl IndexedList {
    // create struct
    fn create_struct(&self, list_name: &Ident, on: &TypePath) -> TokenStream {
        let fields = self.indices.to_declare_struct_field_tokens();

        quote! (
            pub struct #list_name<'a> {
                #(#fields)*
                _items: fast_forward::collections::ro::Slice<'a, #on>,
            }
        )
    }

    // create impls for borrowed and owned
    fn impl_new(&self, list_name: &Ident, on: &TypePath) -> TokenStream {
        let init_fields = self.indices.to_init_struct_field_tokens(&self.on);

        quote! (
            impl<'a> #list_name<'a> {
                pub fn borrowed(slice: &'a [#on]) -> Self {
                    use fast_forward::index::Store;

                    Self {
                        #(#init_fields)*
                        _items: fast_forward::collections::ro::Slice(std::borrow::Cow::Borrowed(slice)),
                    }
                }

                pub fn owned(slice: Vec<#on>) -> Self {
                    use fast_forward::index::Store;

                    Self {
                        #(#init_fields)*
                        _items: fast_forward::collections::ro::Slice(std::borrow::Cow::Owned(slice)),
                    }
                }
            }
        )
    }

    // retrieve method per store
    fn retrieve(&self, list_name: &Ident, on: &TypePath) -> TokenStream {
        let retrieves = self.indices.to_retrieve_tokens(on);

        quote!(
            impl<'a> #list_name<'a> {
                #(#retrieves)*
            }
        )
    }

    // impl `std::ops::Index` trait
    fn impl_index(&self, list_name: &Ident, on: &TypePath) -> TokenStream {
        quote!(
            impl std::ops::Index<usize> for #list_name<'_> {
                type Output = #on;

                fn index(&self, pos: usize) -> &Self::Output {
                    &self._items[pos]
                }
            }
        )
    }

    // impl `std::ops::Deref` trait
    fn impl_deref(&self, list_name: &Ident, on: &TypePath) -> TokenStream {
        quote!(
            impl<'a> std::ops::Deref for #list_name<'a> {
                type Target = [#on];

                fn deref(&self) -> &Self::Target {
                    &self._items.0
                }
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
pub(crate) enum Kind {
    /// Read only
    RO,
    /// Read write
    RW,
    /// Read write delete
    RWD,
}

impl Parse for Kind {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(keyword::ro) {
            input.parse::<keyword::ro>()?;
            Ok(Kind::RO)
        } else if input.peek(keyword::rw) {
            input.parse::<keyword::rw>()?;
            Ok(Kind::RW)
        } else if input.peek(keyword::rwd) {
            input.parse::<keyword::rwd>()?;
            Ok(Kind::RWD)
        } else {
            // default, if no kind find
            Ok(Kind::RO)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::Index;
    use crate::list::IndexedList;
    use syn::parse_quote;

    #[test]
    fn create_struct() {
        let list_name = Ident::new("Cars", proc_macro2::Span::call_site());
        let on = syn::parse_str::<TypePath>("Car").unwrap();

        let l = syn::parse_str::<IndexedList>("create rw Cars on Car using {}").unwrap();
        let ts = l.create_struct(&list_name, &on);

        let ts2: TokenStream = parse_quote!(
            pub struct #list_name<'a> {
                _items: fast_forward::collections::ro::Slice<'a, #on>,
            }
        );

        assert_eq!(ts.to_string(), ts2.to_string());
    }

    #[test]
    fn kind() {
        assert_eq!(Kind::RO, syn::parse_str::<Kind>("ro").unwrap());
        assert_eq!(Kind::RW, syn::parse_str::<Kind>("rw").unwrap());
        assert_eq!(Kind::RWD, syn::parse_str::<Kind>("rwd").unwrap());

        assert_eq!(Kind::RO, syn::parse_str::<Kind>("").unwrap());
    }

    #[test]
    fn list() {
        let idx = syn::parse_str::<Index>("id: UIntIndex => 0").unwrap();

        assert_eq!(
            IndexedList {
                name: Ident::new("Cars", proc_macro2::Span::call_site()),
                kind: Kind::RW,
                on: syn::parse_str::<TypePath>("Car").unwrap(),
                indices: Indices(vec![idx]),
            },
            syn::parse_str::<IndexedList>(
                "create rw Cars on Car using {
                id: UIntIndex => 0,
            }"
            )
            .unwrap()
        );
    }

    #[test]
    fn empty_list_default_kind() {
        assert_eq!(
            IndexedList {
                name: Ident::new("Cars", proc_macro2::Span::call_site()),
                kind: Kind::RO,
                on: syn::parse_str::<TypePath>("mymod::Car").unwrap(),
                indices: Indices(vec![]),
            },
            syn::parse_str::<IndexedList>("create Cars on mymod::Car using {}").unwrap()
        );
    }

    #[test]
    fn list_err_kw() {
        assert_eq!(
            "expected `create`",
            syn::parse_str::<IndexedList>("crea Cars on Car")
                .unwrap_err()
                .to_string()
        );
        assert_eq!(
            "expected `on`",
            syn::parse_str::<IndexedList>("create Cars onn Car")
                .unwrap_err()
                .to_string()
        );
    }

    #[test]
    fn list_err_ident() {
        assert_eq!(
            "expected identifier",
            syn::parse_str::<IndexedList>("create 9Cars")
                .unwrap_err()
                .to_string()
        );
    }
}
