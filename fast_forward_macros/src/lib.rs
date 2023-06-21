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
use quote::ToTokens;
use syn::parse_macro_input;

#[proc_macro]
pub fn create_indexed_list(input: TokenStream) -> TokenStream {
    let list = parse_macro_input!(input as IndexedList);
    TokenStream::from(list.into_token_stream())
}
