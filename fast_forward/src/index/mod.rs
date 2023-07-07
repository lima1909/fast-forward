//! The purpose of an Index is to find faster a specific item in a list (Slice, Vec, ...).
//! This means, it does not have to touch and compare every item in the list.
//!
//! An Index has two parts, a `Key` (item to search for) and a position (the index in the list) `Index`.
//!
//! There are two types of Index:
//! - `Unique Index`: for a given `Key` exist exactly one Index.
//! - `Multi Index` : for a given `Key` exists many Indices.
//!
//! # Example for an Vec-Multi-Index:
//!
//! Map-Index:
//!
//! - `Key`   = name (String)
//! - `Index` = index is the position in a List (Vec)
//!
//! ```text
//! let _names = vec!["Paul", "Jasmin", "Inge", "Paul", ...];
//!
//!  Key       | Index
//! -------------------
//!  "Jasmin"  | 1
//!  "Paul"    | 0, 3
//!  "Inge"    | 2
//!   ...      | ...
//! ```
pub mod indices;
pub mod map;
pub mod ops;
pub mod store;
pub mod uint;
pub mod view;

#[cfg(test)]
pub(crate) mod filter {
    use super::{indices::Indices, store::Filterable};

    /// Wrapper for an given [`Filterable`] implementation.
    /// The Index-slice (&[usize]), will also be wrapped in the [`Indices`] implementation.
    #[repr(transparent)]
    pub struct Filter<'f, F>(pub &'f F);

    impl<'f, F> Filter<'f, F>
    where
        F: Filterable,
    {
        #[inline]
        pub fn eq(&self, key: &F::Key) -> Indices<'f, F::Index>
        where
            F::Index: Clone,
        {
            Indices::from_sorted_slice(self.0.get(key))
        }

        #[inline]
        pub fn contains(&self, key: &F::Key) -> bool {
            self.0.contains(key)
        }
    }
}
