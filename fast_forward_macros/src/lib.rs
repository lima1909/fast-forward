//! # Macro for creating mulit-index collections.
//!
//! The description and an example can you find by the [`fast()`] macro.
//!

mod index;
mod list;

use crate::list::IndexedList;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse_macro_input;

/// Macro, which create a struct for a `Multi-Indexed-Collections`.
///
/// ## `kind` of a collection:
/// - `ro`: read only (default)
/// - `rw`: read write
/// - `rwd`: read write delete
///
/// ## `type` of a collection:
/// - `list`: Vec, Array, VecDeque, ...
/// - `ref_list`: &Vec, &\[T\], ...
/// - `map`: HashMap, BTreeMap, ...
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
