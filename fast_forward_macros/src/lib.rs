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
//!
//! struct Cars<'c> {
//!     ids: ROIndexList<'c, Car, UIntIndex>,
//!     names: ROIndexList<'c, Car, MapIndex>,
//! }
//! ´´´

mod index;
mod list;

use crate::list::IndexedList;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Ident, Member, TypePath};

#[proc_macro]
pub fn create_indexed_list(input: TokenStream) -> TokenStream {
    let list = parse_macro_input!(input as IndexedList);
    let list: ToTokensList = list.into();
    TokenStream::from(list.into_token_stream())
}

struct ToTokensList {
    name: Ident,
    on: TypePath,
    indices: Vec<Index>,
}

// ids: ROIndexList<'c, Car, UIntIndex>,

impl ToTokens for ToTokensList {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let list_name = self.name.clone();
        let on = self.on.clone();

        let indices = self
            .indices
            .iter()
            .map(|i| i.to_token_stream())
            .collect::<Vec<_>>();

        dbg!(&indices);

        let indices_names = self
            .indices
            .iter()
            .map(|i| i.name.clone())
            .collect::<Vec<_>>();

        dbg!(&indices_names);

        tokens.extend(quote! {

            pub struct #list_name<'a> {
                #(#indices)*
            }

            impl<'a> #list_name<'a> {
                pub fn borrowed(slice: &[#on]) -> Self {
                    // Self {
                        // #(#indices.names: fast_forward::collections::ro::ROIndexList::<'_, _, #indices.strukt>::borrowed(|o: &#on| #indices.field, slice),)*
                    // }
                    todo!()
                }
            }

        });
    }
}

struct Index {
    name: Ident,
    store: TypePath,
    #[allow(dead_code)]
    field: Member,
    strukt: TypePath,
}

impl ToTokens for Index {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = self.name.clone();
        let store = self.store.clone();
        let strukt = self.strukt.clone();

        tokens.extend(quote! {
            #name: fast_forward::collections::ro::ROIndexList<'a, #strukt, #store>,
        });
    }
}
impl From<IndexedList> for ToTokensList {
    fn from(from: IndexedList) -> Self {
        let mut result = ToTokensList {
            name: from.name,
            on: from.on.clone(),
            indices: vec![],
        };
        for idx in from.indices.0 {
            result.indices.push(Index {
                name: idx.name,
                store: idx.store,
                strukt: from.on.clone(),
                field: idx.field.clone(),
            })
        }
        result
    }
}
