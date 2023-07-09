//! ```text
//! create [ro | rw | rwd] [list | ref_list | map] Cars on Car
//! kw     Kind            name kw on(type)
//!
//! List {
//!     name: Ident(Cars)
//!     kind: Kind::RO,
//!     typ:  Type::List,
//!     on:   Type(Car),
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

    // Typ
    custom_keyword!(list);
    custom_keyword!(ref_list);
    custom_keyword!(map);
}

/// create [ro | rw | rwd] [list | ref_list | map] Cars on Car
/// kw     Kind            type                    name kw on(type)
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct IndexedList {
    pub(crate) name: Ident,
    pub(crate) kind: Kind,
    pub(crate) typ: Type,
    pub(crate) on: TypePath,
    pub(crate) indices: Indices,
}

impl Parse for IndexedList {
    fn parse(input: ParseStream) -> Result<Self> {
        // create
        let _kw_create = input.parse::<keyword::create>()?;
        // kind: [ro | rw | rwd]
        let kind = input.parse::<Kind>()?;
        // type: [list | ref_list | map]
        let typ = input.parse::<Type>()?;
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
            typ,
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
        tokens.extend(self.retrieve(&list_name));
        tokens.extend(self.impl_deref(&list_name));
    }
}

impl IndexedList {
    // create struct
    fn create_struct(&self, list_name: &Ident, on: &TypePath) -> TokenStream {
        let fields = self.indices.to_declare_struct_field_tokens();

        match self.typ {
            Type::List => {
                quote! (
                    pub struct #list_name<L = Vec<#on>> {
                        #(#fields)*
                        items: L,
                    }
                )
            }
            Type::RefList => {
                quote! (
                    pub struct #list_name<'a> {
                        #(#fields)*
                        items: fast_forward::collections::ro::Slice<'a, #on>,
                    }
                )
            }
            Type::Map => {
                quote! (
                    pub struct #list_name<X, M = HashMap<X, #on>> {
                        #(#fields)*
                        items: M,
                        _idx: std::marker::PhantomData<X>,
                    }
                )
            }
        }
    }

    // create impls for borrowed and owned
    fn impl_new(&self, list_name: &Ident, on: &TypePath) -> TokenStream {
        let init_fields = self.indices.to_init_struct_field_tokens(&self.on);

        match self.typ {
            Type::List => {
                quote! (
                    impl<L> #list_name<L>
                    where
                        L: std::ops::Index<usize, Output = #on>,
                    {
                        pub fn new(items: L) -> Self
                        where
                            L: fast_forward::index::store::ToStore<usize, #on>,
                        {
                            Self {
                                #(#init_fields)*
                                items,
                            }
                        }
                    }
                )
            }
            Type::RefList => {
                quote! (
                    impl<'a> #list_name<'a> {
                        pub fn new(items: &'a [#on]) -> Self {
                            use fast_forward::index::store::ToStore;

                            Self {
                                #(#init_fields)*
                                items: fast_forward::collections::ro::Slice(items),
                            }
                        }
                    }
                )
            }
            Type::Map => {
                quote! (
                    impl<X, M> #list_name<X, M>
                    where
                        S: Store<Index = X>,
                        M: Index<X>,
                                    {
                        pub fn new(items: L) -> Self
                        where
                            S: Store<Key = K, Index = X>,
                            X: Eq + Hash + Clone,
                            M: fast_forward::index::store::ToStore<X, #on>,

                        {
                            Self {
                                #(#init_fields)*
                                items,
                                _idx:  std::marker::PhantomData<X>,
                            }
                        }
                    }
                )
            }
        }
    }

    // retrieve method per store
    fn retrieve(&self, list_name: &Ident) -> TokenStream {
        let retrieves = self.indices.to_retrieve_tokens(&self.typ, &self.on);

        match self.typ {
            Type::List => {
                quote!(
                    impl<L> #list_name<L> {
                        #(#retrieves)*
                    }
                )
            }
            Type::RefList => {
                quote!(
                    impl<'a> #list_name<'a> {
                        #(#retrieves)*
                    }
                )
            }
            Type::Map => {
                quote!(
                    impl<X, M> #list_name<X, M> {
                        #(#retrieves)*
                    }
                )
            }
        }
    }

    // impl `std::ops::Deref` trait
    fn impl_deref(&self, list_name: &Ident) -> TokenStream {
        let on = self.on.clone();

        match self.typ {
            Type::List => {
                quote!(
                    impl<L> std::ops::Deref for #list_name<L> {
                        type Target = L;

                        fn deref(&self) -> &Self::Target {
                            &self.items
                        }
                    }
                )
            }
            Type::RefList => {
                quote!(
                    impl<'a> std::ops::Deref for #list_name<'a> {
                        type Target = [#on];

                        fn deref(&self) -> &Self::Target {
                            self.items.0
                        }
                    }
                )
            }
            Type::Map => {
                quote!(
                    impl<X, M> std::ops::Deref for #list_name<X, M> {
                        type Target = M;

                        fn deref(&self) -> &Self::Target {
                            &self.items
                        }
                    }
                )
            }
        }
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Type {
    /// IList
    List,
    /// IRefList
    RefList,
    /// IMap
    Map,
}

impl Parse for Type {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(keyword::list) {
            input.parse::<keyword::list>()?;
            Ok(Type::List)
        } else if input.peek(keyword::ref_list) {
            input.parse::<keyword::ref_list>()?;
            Ok(Type::RefList)
        } else if input.peek(keyword::map) {
            input.parse::<keyword::map>()?;
            Ok(Type::Map)
        } else {
            // default, if no types find
            Ok(Type::List)
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
    fn create_struct_list() {
        let list_name = Ident::new("Cars", proc_macro2::Span::call_site());
        let on = syn::parse_str::<TypePath>("Car").unwrap();

        let l = syn::parse_str::<IndexedList>("create rw Cars on Car using {}").unwrap();
        let ts = l.create_struct(&list_name, &on);

        let ts2: TokenStream = parse_quote!(
            pub struct #list_name<L = Vec<Car>> {
                items: L,
            }
        );

        assert_eq!(ts.to_string(), ts2.to_string());
    }

    #[test]
    fn create_struct_ref_list() {
        let list_name = Ident::new("Cars", proc_macro2::Span::call_site());
        let on = syn::parse_str::<TypePath>("Car").unwrap();

        let l = syn::parse_str::<IndexedList>("create rw ref_list Cars on Car using {}").unwrap();
        let ts = l.create_struct(&list_name, &on);

        let ts2: TokenStream = parse_quote!(
            pub struct #list_name<'a> {
                items: fast_forward::collections::ro::Slice<'a, #on>,
            }
        );

        assert_eq!(ts.to_string(), ts2.to_string());
    }

    #[test]
    fn create_struct_map() {
        let list_name = Ident::new("Cars", proc_macro2::Span::call_site());
        let on = syn::parse_str::<TypePath>("Car").unwrap();

        let l = syn::parse_str::<IndexedList>("create rw map Cars on Car using {}").unwrap();
        let ts = l.create_struct(&list_name, &on);

        let ts2: TokenStream = parse_quote!(
            pub struct #list_name<X, M = HashMap<X, Car>> {
                items: M,
                _idx:  std::marker::PhantomData<X>,
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
    fn types() {
        assert_eq!(Type::List, syn::parse_str::<Type>("list").unwrap());
        assert_eq!(Type::RefList, syn::parse_str::<Type>("ref_list").unwrap());
        assert_eq!(Type::Map, syn::parse_str::<Type>("map").unwrap());

        assert_eq!(Type::List, syn::parse_str::<Type>("").unwrap());
    }

    #[test]
    fn list() {
        let idx = syn::parse_str::<Index>("id: UIntIndex => 0").unwrap();

        assert_eq!(
            IndexedList {
                name: Ident::new("Cars", proc_macro2::Span::call_site()),
                kind: Kind::RW,
                typ: Type::RefList,
                on: syn::parse_str::<TypePath>("Car").unwrap(),
                indices: Indices(vec![idx]),
            },
            syn::parse_str::<IndexedList>(
                "create rw ref_list Cars on Car using {
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
                typ: Type::List,
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
