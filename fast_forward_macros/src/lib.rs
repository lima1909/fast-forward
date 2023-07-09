//! # Grammer for creating an Indexed List.
//!
//! ```text
//! create [ro [default] | rw | rwd] [indexed-list-name] on [struct] using {
//!     [index-name]: [store-impl] => [struct-field]
//! }
//! ```
//! - `ro`: read only (default)
//! - `rw`: read write
//! - `rwd`: read writ delete
//!
//! ## Example:
//!
//! ```text
//! use fast_forward_macros::fast;
//!
//! #[derive(Clone)]
//! pub struct Car(usize, String);
//!
//! fast!(
//!     create ro Cars on Car using {
//!         id:   fast_forward::index::uint::UIntIndex => 0,
//!         name: fast_forward::index::map::MapIndex   => 1.clone,
//!     }
//! );
//! ´´´
//!

mod index;
mod list;

use crate::list::IndexedList;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse_macro_input;

/// Macro, which create the struct for an `Indexed Collections`.
///
/// ## Collections has the following kinds:
/// - ro: read only (default)
/// - rw: read write
/// - rwd: read write delete
///
/// ## Collections of the Items has one of the following types:
/// - list -> IList (Vec, Array, VecDeque, ...)
/// - ref_list -> IRefList (&Vec, &[T], ...)
/// - map -> IMap (HashMap, BTreeMap, ...)
///
/// ## Example
///
/// ```
/// use fast_forward_macros::fast;
///
/// #[derive(Clone)]
/// pub struct Car(usize, String);
///
/// fast!(
///     create ro ref_list Cars on Car using {
///         id:   fast_forward::index::uint::UIntIndex => 0,
///         name: fast_forward::index::map::MapIndex   => 1.clone,
///     }
/// );
/// ```
#[proc_macro]
pub fn fast(input: TokenStream) -> TokenStream {
    let list = parse_macro_input!(input as IndexedList);
    TokenStream::from(list.into_token_stream())
}
