//! # Grammer for creating an Indexed List (like SQL).
//!
//! ```text
//! create [ro | rw | rwd] [name] on [struct] using {
//!     [index-name]: [store-impl] => [struct-field]
//! }
//! from [borrowed | owned] [slice]
//! ```
//!
//! ## Example:
//!
//! ```text
//! #[derive(Debug, Eq, PartialEq, Clone)]
//! pub struct Car(usize, String);
//!
//! create ro Cars on Car using {
//!     id:   UIntIndex => pk,
//!     name: MapIndex  => name.clone,
//! }
//! from [borrowed] &vec![...]
//! ´´´

mod index;
mod list;

use std::{fmt::Display, str::FromStr};

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::Parse, parse_macro_input, Ident};

#[proc_macro]
pub fn create_indexed_list(input: TokenStream) -> TokenStream {
    let list = parse_macro_input!(input as IndexedList);
    TokenStream::from(list.into_token_stream())
}

#[derive(Debug, Clone, PartialEq)]
struct IndexedList {
    kind: Kind,
    list_name: Ident,
}

impl Parse for IndexedList {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Err(syn::parse::Error::new(
                input.span(),
                "expected keyword: 'create', found empty input",
            ));
        }

        // create
        let _create = Keyword::Create.from_ident(Ident::parse(input)?)?;
        // ro, rw or rwd
        let kind = Kind::parse(input)?;
        // IndexedList-name: Cars
        let list_name = Ident::parse(input)?;

        Ok(Self { kind, list_name })
    }
}

impl ToTokens for IndexedList {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let list_name = self.list_name.clone();

        tokens.extend(quote! {
            #[derive(Debug)]
            pub struct #list_name;

        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)] // TODO: remove this
enum Keyword {
    Create,
    On,
    Using,
    From,
}

impl Keyword {
    #[allow(clippy::wrong_self_convention)]
    fn from_ident(&self, ident: Ident) -> syn::Result<Self> {
        if ident.eq(self) {
            return Ok(*self);
        }

        Err(syn::parse::Error::new(
            ident.span(),
            format!("expected keyword: '{}', found: '{ident}'", self),
        ))
    }
}

impl PartialEq<Keyword> for Ident {
    fn eq(&self, keyword: &Keyword) -> bool {
        self.to_string()
            .to_lowercase()
            .eq(keyword.to_string().as_str())
    }
}

impl Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Keyword::Create => write!(f, "create"),
            Keyword::On => write!(f, "on"),
            Keyword::Using => write!(f, "using"),
            Keyword::From => write!(f, "from"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
enum Kind {
    /// Read only
    RO,
    /// Read write
    RW,
    /// Read write delete
    RWD,
}

impl FromStr for Kind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ro" => Ok(Kind::RO),
            "rw" => Ok(Kind::RW),
            "rwd" => Ok(Kind::RWD),
            _ => Err(format!("invalid index kind: '{s}'. use: ro, rw or rwd")),
        }
    }
}

impl Parse for Kind {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = Ident::parse(input)?;
        Kind::from_str(ident.to_string().as_str())
            .map_err(|msg| syn::parse::Error::new(input.span(), msg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_indexed_list() {
        assert_eq!(
            IndexedList {
                kind: Kind::RO,
                list_name: Ident::new("Cars", proc_macro2::Span::call_site())
            },
            syn::parse_str::<IndexedList>("Create RO Cars").unwrap()
        );
    }

    #[test]
    fn parse_indexed_list_err_empty() {
        assert_eq!(
            "expected keyword: 'create', found empty input",
            syn::parse_str::<IndexedList>(" ").unwrap_err().to_string()
        );
    }

    #[test]
    fn parse_indexed_list_err_no_kw_create() {
        assert_eq!(
            "expected keyword: 'create', found: 'foo'",
            syn::parse_str::<IndexedList>(" foo ro")
                .unwrap_err()
                .to_string()
        );
    }

    #[test]
    fn parse_keyword() {
        assert_eq!(
            Keyword::Create,
            Keyword::Create
                .from_ident(syn::parse_str::<Ident>("create").unwrap())
                .unwrap()
        );
        assert_eq!(
            Keyword::From,
            Keyword::From
                .from_ident(syn::parse_str::<Ident>("from").unwrap())
                .unwrap()
        );
        assert_eq!(
            Keyword::Using,
            Keyword::Using
                .from_ident(syn::parse_str::<Ident>("using").unwrap())
                .unwrap()
        );
        assert_eq!(
            Keyword::On,
            Keyword::On
                .from_ident(syn::parse_str::<Ident>("on").unwrap())
                .unwrap()
        );
    }

    #[test]
    fn parse_keyword_err() {
        assert_eq!(
            "expected keyword: 'on', found: 'foo'",
            Keyword::On
                .from_ident(syn::parse_str::<Ident>("foo").unwrap())
                .unwrap_err()
                .to_string()
        );
    }

    #[test]
    fn parse_kind() {
        assert_eq!(Kind::RO, syn::parse_str::<Kind>("ro").unwrap());
        assert_eq!(Kind::RW, syn::parse_str::<Kind>("Rw").unwrap());
        assert_eq!(Kind::RWD, syn::parse_str::<Kind>("rwD").unwrap());
    }

    #[test]
    fn parse_kind_err() {
        assert_eq!(
            "invalid index kind: 'foo'. use: ro, rw or rwd",
            syn::parse_str::<Kind>("foo").unwrap_err().to_string()
        );
    }
}
