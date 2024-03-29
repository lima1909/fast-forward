//! The `index `module contains the structure for saving and accessing the `Index` implementations.
pub mod imap;
pub mod indices;
pub mod ivec;
pub mod ops;
pub mod store;

pub use imap::MapIndex;
pub use ivec::int::{MultiIntIndex, UniqueIntIndex};
pub use ivec::uint::{MultiUIntIndex, UniqueUIntIndex};

use crate::index::{indices::Indices, store::Filterable};

/// [`Filter`] combines a given [`Filterable`] with the given list of items.
pub struct Filter<'a, F, I> {
    pub(crate) filter: &'a F,
    pub(crate) items: &'a I,
}

impl<'a, F, I> Filter<'a, F, I>
where
    F: Filterable,
{
    pub const fn new(filter: &'a F, items: &'a I) -> Self {
        Self { filter, items }
    }

    #[inline]
    pub fn eq(&self, key: &F::Key) -> Indices<'a, F::Index>
    where
        F::Index: Clone,
    {
        Indices::from_sorted_slice(self.filter.get(key))
    }

    #[inline]
    pub fn contains(&self, key: &F::Key) -> bool {
        self.filter.contains(key)
    }

    #[inline]
    pub fn items(
        &'a self,
        key: &F::Key,
    ) -> impl Iterator<Item = &'a <I as Indexable<F::Index>>::Output>
    where
        I: Indexable<F::Index>,
    {
        self.items.items(self.filter.get(key).iter())
    }
}

/// [`Indexable`] means a collection (Map, Vec, Array, ...)
/// where Items are accessable via an Index.
/// It is a replacement of [`std::ops::Index`].
pub trait Indexable<Idx> {
    type Output;

    /// Get the Item based on the given Index.
    ///
    /// #Panic
    ///
    /// If no Item exist for the given Index.
    fn item(&self, idx: &Idx) -> &Self::Output;

    /// Return an `Iterator` with all `Items`
    /// for a given `Iterator` with `Indices`.
    fn items<'a, I>(&'a self, indices: I) -> Items<Self, Idx, I>
    where
        I: Iterator<Item = &'a Idx>,
        Self: Sized,
    {
        Items {
            items: self,
            indices,
        }
    }
}

/// `Itmes`is an `Iterator` which is created by the `Indexable` trait.
/// `Items` contains all `Items` for a given amount of `Indices`.
pub struct Items<'a, It, X, Idx>
where
    It: Indexable<X>,
    Idx: Iterator<Item = &'a X>,
    X: 'a,
{
    items: &'a It,
    indices: Idx,
}

impl<'a, It, X, Idx> Iterator for Items<'a, It, X, Idx>
where
    It: Indexable<X>,
    It::Output: 'a,
    Idx: Iterator<Item = &'a X>,
    X: 'a,
{
    type Item = &'a It::Output;

    fn next(&mut self) -> Option<Self::Item> {
        self.indices.next().map(|idx| self.items.item(idx))
    }
}

macro_rules! list_indexable {
    ( $( $t:ty ),* ) => {
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

list_indexable!(Vec<T>, std::collections::VecDeque<T>, &[T]);

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
        &self[idx]
    }
}

impl<X, T> Indexable<X> for std::collections::BTreeMap<X, T>
where
    X: Ord + Eq + Hash + Clone + Borrow<X>,
{
    type Output = T;

    fn item(&self, idx: &X) -> &Self::Output {
        &self[idx]
    }
}

#[cfg(feature = "hashbrown")]
impl<X, T> Indexable<X> for hashbrown::HashMap<X, T>
where
    X: Eq + Hash + Clone + Borrow<X>,
{
    type Output = T;

    fn item(&self, idx: &X) -> &Self::Output {
        &self[idx]
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
