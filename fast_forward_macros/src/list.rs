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
        let fields = self.indices.to_field_declare_tokens(&self.on);

        // create struct with declared fields
        tokens.extend(quote! {

                pub struct #list_name<'a> {
                    #(#fields)*
                }

        });

        // create impl for creating the indexed list
        let on = self.on.clone();
        let init_fields = self.indices.to_init_struct_field_tokens(&self.on);

        tokens.extend(quote! {

            impl<'a> #list_name<'a> {
                pub fn borrowed(slice: &'a [#on]) -> Self {
                    Self {
                        #(#init_fields)*
                    }
                }
            }

        });
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
