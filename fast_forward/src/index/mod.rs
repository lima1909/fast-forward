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
pub mod map;
pub mod uint;

use crate::{Iter, ListIndexFilter, EMPTY_IDXS};
use std::borrow::Cow;

/// A Store is a mapping from a given `Key` to one or many `Indices`.
pub trait Store: Default {
    type Key;

    /// Insert an `Key` for a given `Index`.
    ///
    /// Before:
    ///     Female | 3,4
    /// `Insert: (Male, 2)`
    /// After:
    ///     Male   | 2
    ///     Female | 3,4
    ///
    /// OR (if the `Key` already exist):
    ///
    /// Before:
    ///     Female | 3,4
    /// `Insert: (Female, 2)`
    /// After:
    ///     Female | 2,3,4
    ///
    fn insert(&mut self, key: Self::Key, idx: usize);

    /// Update means: `Key` changed, but `Index` stays the same
    ///
    /// Before:
    ///     Male   | 1,2,5  
    ///     Female | 3,4
    /// `Update: (Male, 2, Female)`
    /// After:
    ///     Male   | 1,5
    ///     Female | 2,3,4
    ///
    /// otherwise (`Key` has exact one `Index`), then remove complete row (`Key` and `Index`).
    ///
    /// Before:
    ///     Male   | 2
    ///     Female | 3,4
    /// `Update: (Male, 2, Female)`
    /// After:
    ///     Female | 2,3,4
    ///
    /// If the old `Key` not exist, then is it a insert with the new `Key`:
    ///
    /// Before:
    ///     Female | 3,4
    /// `Update: (Male, 2, Female)`
    /// After:
    ///     Female | 2,3,4
    fn update(&mut self, old_key: Self::Key, idx: usize, new_key: Self::Key) {
        self.delete(old_key, idx);
        self.insert(new_key, idx);
    }

    /// Delete means: if an `Key` has more than one `Index`, then remove only this `Index`:
    ///
    /// Before:
    ///     Male   | 1,2,5  
    ///     Female | 3,4
    /// `Delete: Male: 2`
    /// After:
    ///     Male   | 1,5
    ///     Female | 3,4
    ///
    /// otherwise (`Key` has exact one `Index`), then remove complete row (`Key` and `Index`).
    ///
    /// Before:
    ///     Male   | 2
    ///     Female | 3,4
    /// `Delete: Male: 2`
    /// After:
    ///     Female | 3,4
    ///
    /// If the `Key` not exist, then is `delete`ignored:
    ///
    /// Before:
    ///     Female | 3,4
    /// `Delete: Male: 2`
    /// After:
    ///     Female | 3,4
    ///
    fn delete(&mut self, key: Self::Key, idx: usize);

    /// To reduce memory allocations can create an `Index-store` with capacity.
    fn with_capacity(capacity: usize) -> Self;

    type Retriever<'a>
    where
        Self: 'a;

    /// Get instances, to provide Store specific read/select operations.
    fn retrieve<'a, I, L>(&'a self, items: &'a L) -> ItemRetriever<'a, Self::Retriever<'a>, L>
    where
        I: 'a,
        L: ListIndexFilter<Item = I> + 'a,
        <Self as Store>::Retriever<'a>: Retriever;
}

/// Empty Meta, if the `Retriever` no meta data supported.
pub struct NoMeta;

impl NoMeta {
    pub const fn has_no_meta_data(&self) -> bool {
        true
    }
}

pub struct EqFilter<'s, R: Retriever>(&'s R);

impl<'s, R: Retriever> EqFilter<'s, R> {
    pub fn eq(&self, key: &R::Key) -> Cow<'s, [usize]> {
        self.0.get(key)
    }

    pub fn eq_many<I>(&self, keys: I) -> Cow<[usize]>
    where
        I: IntoIterator<Item = R::Key>,
    {
        self.0.get_many(keys)
    }

    pub fn contains(&self, key: &R::Key) -> bool {
        self.0.contains(key)
    }
}

/// Trait for read/select method from a `Store`.
pub trait Retriever {
    type Key;

    /// Get all indices for a given `Key`.
    fn get(&self, key: &Self::Key) -> Cow<[usize]>;

    /// Combined all given `keys` with an logical `OR`.
    ///
    /// ## Example:
    ///```text
    /// get_many([2, 5, 6]) => get(2) OR get(5) OR get(6)
    /// get_many(2..6]) => get(2) OR get(3) OR get(4) OR get(5)
    /// ```
    fn get_many<I>(&self, keys: I) -> Cow<[usize]>
    where
        I: IntoIterator<Item = Self::Key>,
    {
        let mut it = keys.into_iter();
        match it.next() {
            Some(key) => {
                let mut c = self.get(&key);
                for k in it {
                    c = crate::query::or(c, self.get(&k))
                }
                c
            }
            None => Cow::Borrowed(EMPTY_IDXS),
        }
    }

    /// Checks whether the `Key` exists.
    fn contains(&self, key: &Self::Key) -> bool {
        !self.get(key).is_empty()
    }

    type Filter<'f>
    where
        Self: 'f;

    /// Return filter methods from the `Store`.
    fn filter<'r, P>(&'r self, predicate: P) -> Cow<[usize]>
    where
        P: Fn(<Self as Retriever>::Filter<'r>) -> Cow<[usize]>;

    type Meta<'m>
    where
        Self: 'm;

    /// Return meta data from the `Store`.
    fn meta(&self) -> Self::Meta<'_>;
}

pub struct ItemRetriever<'a, R, L> {
    inner: &'a R,
    items: &'a L,
}

impl<'a, R, L> ItemRetriever<'a, R, L>
where
    R: Retriever,
    L: ListIndexFilter,
{
    /// Get all items for a given `Key`.
    pub fn get(&self, key: &R::Key) -> Iter<'a, L> {
        let indices = self.inner.get(key);
        self.items.filter(indices)
    }

    /// Combined all given `keys` with an logical `OR`.
    ///
    /// ## Example:
    ///```text
    /// get_many([2, 5, 6]) => get(2) OR get(5) OR get(6)
    /// get_many(2..6]) => get(2) OR get(3) OR get(4) OR get(5)
    /// ```
    pub fn get_many<I>(&self, keys: I) -> Iter<'a, L>
    where
        I: IntoIterator<Item = R::Key>,
    {
        let indices = self.inner.get_many(keys);
        self.items.filter(indices)
    }

    /// Checks whether the `Key` exists.
    pub fn contains(&self, key: R::Key) -> bool {
        !self.inner.get(&key).is_empty()
    }

    /// Return filter methods from the `Store`.
    pub fn filter<P>(&self, predicate: P) -> Iter<'a, L>
    where
        P: Fn(R::Filter<'a>) -> Cow<[usize]>,
    {
        let indices = self.inner.filter(predicate);
        self.items.filter(indices)
    }

    /// Return meta data from the `Store`.
    pub fn meta(&self) -> R::Meta<'_> {
        self.inner.meta()
    }
}

#[derive(Debug, Default)]
struct MinMax<K> {
    min: K,
    max: K,
}

impl<K: Default + Ord> MinMax<K> {
    fn new_min(&mut self, key: K) -> &K {
        if self.min == K::default() || self.min > key {
            self.min = key
        }
        &self.min
    }

    fn new_max(&mut self, key: K) -> &K {
        if self.max < key {
            self.max = key
        }
        &self.max
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Index(Vec<usize>);

impl Index {
    #[inline]
    pub fn new(idx: usize) -> Self {
        Self(vec![idx])
    }

    #[inline]
    pub fn add(&mut self, idx: usize) {
        if let Err(pos) = self.0.binary_search(&idx) {
            self.0.insert(pos, idx);
        }
    }

    #[inline]
    pub fn get(&self) -> Cow<[usize]> {
        Cow::Borrowed(&self.0)
    }

    #[inline]
    pub fn remove(&mut self, idx: usize) -> Cow<[usize]> {
        self.0.retain(|v| v != &idx);
        self.get()
    }

    pub fn or<'a>(&'a self, rhs: Cow<'a, [usize]>) -> Cow<'a, [usize]> {
        crate::query::or(self.get(), rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_unique() {
        let u = Index::new(0);
        assert_eq!([0], *u.get());
    }

    #[test]
    fn index_multi() {
        let mut m = Index::new(2);
        assert_eq!([2], *m.get());

        m.add(1);
        assert_eq!([1, 2], *m.get());
    }

    #[test]
    fn index_multi_duplicate() {
        let mut m = Index::new(1);
        assert_eq!([1], *m.get());

        // ignore add: 1, 1 exists already
        m.add(1);
        assert_eq!([1], *m.get());
    }

    #[test]
    fn index_multi_ordered() {
        let mut m = Index::new(5);
        assert_eq!([5], *m.get());

        m.add(3);
        m.add(1);
        m.add(4);

        assert_eq!([1, 3, 4, 5], *m.get());
    }

    #[test]
    fn index_container_multi() {
        let mut lhs = Index::new(5);
        lhs.add(3);
        lhs.add(2);
        lhs.add(4);

        let mut rhs = Index::new(5);
        rhs.add(2);
        rhs.add(9);

        assert_eq!([2, 3, 4, 5, 9], *lhs.or(rhs.get()));
    }

    #[test]
    fn index_container_unique() {
        let mut lhs = Index::new(5);

        let rhs = Index::new(5);
        assert_eq!([5], *lhs.or(rhs.get()));

        lhs.add(0);
        assert_eq!([0, 5], *lhs.or(rhs.get()));
    }

    #[test]
    fn index_remove() {
        let mut pos = Index::new(5);
        assert_eq!([5], *pos.get());

        assert!(pos.remove(5).is_empty());
        // double remove
        assert!(pos.remove(5).is_empty());

        let mut pos = Index::new(5);
        pos.add(2);
        assert_eq!([2], *pos.remove(5));

        let mut pos = Index::new(5);
        pos.add(2);
        assert_eq!([5], *pos.remove(2));
    }

    #[test]
    fn min() {
        assert_eq!(0, MinMax::default().min);
        assert_eq!(&0, MinMax::default().new_min(0));
        assert_eq!(&1, MinMax::default().new_min(1));

        let mut min = MinMax::default();
        min.new_min(1);
        min.new_min(0);
        assert_eq!(0, min.min);

        let mut min = MinMax::default();
        min.new_min(1);
        min.new_min(2);
        assert_eq!(1, min.min);

        let mut min = MinMax::default();
        min.new_min(2);
        min.new_min(1);
        assert_eq!(1, min.min);
    }

    #[test]
    fn max() {
        assert_eq!(0, MinMax::default().max);
        assert_eq!(&0, MinMax::default().new_max(0));
        assert_eq!(&1, MinMax::default().new_max(1));

        let mut max = MinMax::default();
        max.new_max(1);
        max.new_max(0);
        assert_eq!(1, max.max);

        let mut max = MinMax::default();
        max.new_max(1);
        max.new_max(2);
        assert_eq!(2, max.max);
    }
}
