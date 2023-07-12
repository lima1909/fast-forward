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

/// [`Indexable`] means a collection (Map, Vec, Array, ...)
/// where Items are accessable via an Index.
/// It is a replacement of [`std::ops::Index`].
pub trait Indexable<Idx> {
    type Output;

    /// Get the Item based on the given Index.
    fn item(&self, idx: &Idx) -> &Self::Output;
}

macro_rules! list_indexable {
    ( $( $t:ty )* ) => {
        $(
        impl<T> Indexable<usize> for $t {
            type Output = T;

            fn item(&self, idx: &usize) -> &Self::Output {
                &self[*idx]
            }
        }
        )*
    };
}

list_indexable!(Vec<T> std::collections::VecDeque<T> &[T]);

impl<T, const N: usize> Indexable<usize> for [T; N] {
    type Output = T;

    fn item(&self, idx: &usize) -> &Self::Output {
        &self[*idx]
    }
}

use std::{borrow::Borrow, hash::Hash};

impl<X, T> Indexable<X> for std::collections::HashMap<X, T>
where
    X: Eq + Hash + Clone + Borrow<X>,
{
    type Output = T;

    fn item(&self, idx: &X) -> &Self::Output {
        self.get(idx).expect("no entry found for key")
    }
}

impl<X, T> Indexable<X> for std::collections::BTreeMap<X, T>
where
    X: Ord + Eq + Hash + Clone + Borrow<X>,
{
    type Output = T;

    fn item(&self, idx: &X) -> &Self::Output {
        self.get(idx).expect("no entry found for key")
    }
}

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
