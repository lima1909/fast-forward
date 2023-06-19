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
use syn::{
    parse::{Parse, ParseStream},
    Ident, Result, TypePath,
};

mod keyword {
    use syn::custom_keyword;

    custom_keyword!(create);
    custom_keyword!(on);
    // Kinds
    custom_keyword!(ro);
    custom_keyword!(rw);
    custom_keyword!(rwd);
}

#[derive(Debug, Clone, PartialEq)]
struct List {
    name: Ident,
    kind: Kind,
    on: TypePath,
}

impl Parse for List {
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

        Ok(List { name, kind, on })
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

    #[test]
    fn kind() {
        assert_eq!(Kind::RO, syn::parse_str::<Kind>("ro").unwrap());
        assert_eq!(Kind::RW, syn::parse_str::<Kind>("rw").unwrap());
        assert_eq!(Kind::RWD, syn::parse_str::<Kind>("rwd").unwrap());

        assert_eq!(Kind::RO, syn::parse_str::<Kind>("").unwrap());
    }

    #[test]
    fn list() {
        assert_eq!(
            List {
                name: Ident::new("Cars", proc_macro2::Span::call_site()),
                kind: Kind::RW,
                on: syn::parse_str::<TypePath>("Car").unwrap(),
            },
            syn::parse_str::<List>("create rw Cars on Car").unwrap()
        );
    }

    #[test]
    fn list_default_kind() {
        assert_eq!(
            List {
                name: Ident::new("Cars", proc_macro2::Span::call_site()),
                kind: Kind::RO,
                on: syn::parse_str::<TypePath>("mymod::Car").unwrap(),
            },
            syn::parse_str::<List>("create Cars on mymod::Car").unwrap()
        );
    }

    #[test]
    fn list_err_kw() {
        assert_eq!(
            "expected `create`",
            syn::parse_str::<List>("crea Cars on Car")
                .unwrap_err()
                .to_string()
        );
        assert_eq!(
            "expected `on`",
            syn::parse_str::<List>("create Cars onn Car")
                .unwrap_err()
                .to_string()
        );
    }

    #[test]
    fn list_err_ident() {
        assert_eq!(
            "expected identifier",
            syn::parse_str::<List>("create 9Cars")
                .unwrap_err()
                .to_string()
        );
    }
}