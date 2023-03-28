//! The purpose of an Index is to find faster a specific item in a list (Slice, Vec, ...).
//! This means, it does not have to touch and compare every item in the list.
//!
//! An Index has two parts, a `Key` (item to search for) and a position (the index in the list) [`Idx`].
//!
//! There are two types of Index:
//! - `Unique Index`: for a given `Key` exist exactly one [`Idx`].
//! - `Multi Index` : for a given `Key` exists many [`Idx`]s.
//!
//! # Example for an Vec-Multi-Index:
//!
//! Map-Index:
//!
//! - `Key` = name (String)
//! - [`Idx`] = index is the position in a List (Vec)
//!
//! ```text
//! let _names = vec!["Paul", "Jasmin", "Inge", "Paul", ...];
//!
//!  Key       | Idx
//! -------------------
//!  "Jasmin"  | 1
//!  "Paul"    | 0, 3
//!  "Inge"    | 2
//!   ...      | ...
//! ```
pub mod map;
pub mod uint;

use crate::{Idx, EMPTY_IDXS};
use std::borrow::Cow;

/// A Store is a mapping from a given `Key` to one or many `Indices`.
pub trait Store<K>: Default {
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
    fn insert(&mut self, key: K, idx: Idx);

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
    /// If the old `Key` not exist, then is it a insert with the new `Key`:
    ///
    /// Before:
    ///     Female | 3,4
    /// `Update: (Male, 2, Female)`
    /// After:
    ///     Female | 2,3,4

    fn update(&mut self, _old_key: K, _idx: Idx, _new_key: K) {}

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
    fn delete(&mut self, _key: K, _idx: Idx) {}
}

pub trait Equals<K> {
    /// Find all `Idx` with the given `Key`.
    fn eq(&self, key: K) -> Cow<[Idx]>;

    /// Combined all given `keys` with an logical `OR`.
    ///
    /// ## Example:
    ///```text
    /// eq_iter([2, 5, 6]) => eq(2) OR eq(5) OR eq(6)
    /// eq_iter(2..6]) => eq(2) OR eq(3) OR eq(4) OR eq(5)
    /// ```
    fn eq_iter<I>(&self, keys: I) -> Cow<[Idx]>
    where
        I: IntoIterator<Item = K>,
    {
        let mut it = keys.into_iter();
        match it.next() {
            Some(key) => {
                let mut c = self.eq(key);
                for k in it {
                    c = crate::query::or(c, self.eq(k))
                }
                c
            }
            None => Cow::Borrowed(EMPTY_IDXS),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Index(Vec<Idx>);

impl Index {
    #[inline]
    pub fn new(idx: Idx) -> Self {
        Self(vec![idx])
    }

    #[inline]
    pub fn add(&mut self, idx: Idx) {
        if let Err(pos) = self.0.binary_search(&idx) {
            self.0.insert(pos, idx);
        }
    }

    #[inline]
    pub fn get(&self) -> Cow<[Idx]> {
        Cow::Borrowed(&self.0)
    }

    pub fn or<'a>(&'a self, rhs: Cow<'a, [Idx]>) -> Cow<'a, [Idx]> {
        crate::query::or(self.get(), rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique() {
        let u = Index::new(0);
        assert_eq!([0], *u.get());
    }

    #[test]
    fn multi() {
        let mut m = Index::new(2);
        assert_eq!([2], *m.get());

        m.add(1);
        assert_eq!([1, 2], *m.get());
    }

    #[test]
    fn multi_duplicate() {
        let mut m = Index::new(1);
        assert_eq!([1], *m.get());

        // ignore add: 1, 1 exists already
        m.add(1);
        assert_eq!([1], *m.get());
    }

    #[test]
    fn multi_ordered() {
        let mut m = Index::new(5);
        assert_eq!([5], *m.get());

        m.add(3);
        m.add(1);
        m.add(4);

        assert_eq!([1, 3, 4, 5], *m.get());
    }

    #[test]
    fn container_multi() {
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
    fn container_unique() {
        let mut lhs = Index::new(5);

        let rhs = Index::new(5);
        assert_eq!([5], *lhs.or(rhs.get()));

        lhs.add(0);
        assert_eq!([0, 5], *lhs.or(rhs.get()));
    }
}
