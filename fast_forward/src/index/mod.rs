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

use std::{borrow::Borrow, collections::HashMap, hash::Hash};

use self::store::Store;

/// Convert an Iterator of Indices to an Iterator of Items.
pub fn to_itmes<'a, It, I>(indices: It, items: &'a I) -> impl Iterator<Item = &'a I::Output> + 'a
where
    It: IntoIterator,
    I: Indexable<It::Item>,
    <It as IntoIterator>::IntoIter: 'a,
{
    indices.into_iter().map(|idx| items.item(&idx))
}

/// [`Indexable`] means a collection (Map, Vec, Array, ...) which can indexed.
/// The collection
/// - must be insertable in the [`crate::index::store::Store`] and
/// - the access of an Item is possible via an Index.
pub trait Indexable<Idx> {
    type Output;

    /// Insert the Items from the collection into the Store.
    fn to_store<S, F>(&self, field: F) -> S
    where
        S: Store<Index = Idx>,
        F: FnMut(&Self::Output) -> S::Key;

    /// Get the Item based on the given Index.
    fn item(&self, idx: &Idx) -> &Self::Output;
}

impl<T> Indexable<usize> for &[T] {
    type Output = T;

    fn to_store<S, F>(&self, field: F) -> S
    where
        S: Store<Index = usize>,
        F: FnMut(&Self::Output) -> S::Key,
    {
        S::from_list(self.iter().map(field))
    }

    fn item(&self, idx: &usize) -> &Self::Output {
        &self[*idx]
    }
}

impl<T, const N: usize> Indexable<usize> for [T; N] {
    type Output = T;

    fn to_store<S, F>(&self, field: F) -> S
    where
        S: Store<Index = usize>,
        F: FnMut(&Self::Output) -> S::Key,
    {
        S::from_list(self.iter().map(field))
    }

    fn item(&self, idx: &usize) -> &Self::Output {
        &self[*idx]
    }
}

impl<T> Indexable<usize> for Vec<T> {
    type Output = T;

    fn to_store<S, F>(&self, field: F) -> S
    where
        S: Store<Index = usize>,
        F: FnMut(&Self::Output) -> S::Key,
    {
        S::from_list(self.iter().map(field))
    }

    fn item(&self, idx: &usize) -> &Self::Output {
        &self[*idx]
    }
}

impl<T> Indexable<usize> for std::collections::VecDeque<T> {
    type Output = T;

    fn to_store<S, F>(&self, field: F) -> S
    where
        S: Store<Index = usize>,
        F: FnMut(&Self::Output) -> S::Key,
    {
        S::from_list(self.iter().map(field))
    }

    fn item(&self, idx: &usize) -> &Self::Output {
        &self[*idx]
    }
}

impl<X, T> Indexable<X> for HashMap<X, T>
where
    X: Eq + Hash + Clone + Borrow<X>,
{
    type Output = T;

    fn to_store<S, F>(&self, mut field: F) -> S
    where
        S: Store<Index = X>,
        F: FnMut(&Self::Output) -> S::Key,
    {
        S::from_map(self.iter().map(|(idx, item)| (field(item), idx.clone())))
    }

    fn item(&self, idx: &X) -> &Self::Output {
        self.get(idx).expect("no entry found for key")
    }
}

impl<X, T> Indexable<X> for std::collections::BTreeMap<X, T>
where
    X: Ord + Eq + Hash + Clone + Borrow<X>,
{
    type Output = T;

    fn to_store<S, F>(&self, mut field: F) -> S
    where
        S: Store<Index = X>,
        F: FnMut(&Self::Output) -> S::Key,
    {
        S::from_map(self.iter().map(|(idx, item)| (field(item), idx.clone())))
    }

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
