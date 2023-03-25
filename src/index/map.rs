//! Indices for string types: ([`str`]).
//!
//! The `Key` is the Hash-Key in the Index-Map ([`StrMapIndex`]).
//!
//!
//!```text
//! let _unique_values = vec!["Paul", "Mario", "Jasmin", ...];
//!
//! Unique Index:
//!
//!  Key      | Idx
//! --------------------
//!  "Jasmin" |  2
//!  "Mario"  |  1
//!  "Paul"   |  0
//!   ...     | ...
//!
//! let _multi_values = vec!["Jasmin", "Mario", "Jasmin", ...];
//!
//! Multi Index:
//!
//!  Key      | Idx
//! --------------------
//!  "Jasmin" |  0, 2
//!  "Mario"  |  1
//!   ...     | ...
//!
//! ```
use crate::{
    index::{Index, Multi, Store, Unique},
    query::EMPTY_IDXS,
    Idx,
};
use std::{
    borrow::Cow,
    collections::{btree_map::Entry, BTreeMap},
    fmt::Debug,
};

/// Unique Key from type [`str`].
pub type UniqueStrIdx<'a> = StrMapIndex<'a, Unique>;

/// An not unique [`str`] Key, which can occur multiple times.
pub type MultiStrIdx<'a> = StrMapIndex<'a, Multi>;

/// `Key` is from type [`str`] and use [`std::collections::BTreeMap`] for the searching.
#[derive(Debug, Default)]
pub struct StrMapIndex<'s, I: Index>(BTreeMap<&'s str, I>);

impl<'s, I: Index> Store<&'s str> for StrMapIndex<'s, I> {
    fn insert(&mut self, k: &'s str, i: Idx) -> super::Result {
        match self.0.entry(k) {
            Entry::Vacant(e) => {
                e.insert(I::new(i));
                Ok(())
            }
            Entry::Occupied(mut e) => e.get_mut().add(i),
        }
    }
}

impl<'s, I: Index> StrMapIndex<'s, I> {
    pub fn eq(&self, key: &'s str) -> Cow<[Idx]> {
        match self.0.get(key) {
            Some(i) => Cow::Borrowed(i.get()),
            None => Cow::Borrowed(EMPTY_IDXS),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use crate::query;

    mod unique {
        use super::*;

        #[test]
        fn empty() {
            let i = UniqueStrIdx::default();
            assert_eq!(0, i.eq("Jasmin").len());
            assert!(i.0.is_empty());
        }

        #[test]
        fn find_idx_2() {
            let mut i = UniqueStrIdx::default();
            i.insert("Jasmin", 4).unwrap();

            assert_eq!(*i.eq("Jasmin"), [4]);
            assert_eq!(1, i.0.len());
        }

        #[test]
        fn or_find_idx_3_4() {
            let mut idx = UniqueStrIdx::default();
            idx.insert("Jasmin", 4).unwrap();
            idx.insert("Mario", 8).unwrap();
            idx.insert("Paul", 6).unwrap();

            let r = query(idx.eq("Mario")).or(idx.eq("Paul")).exec();
            assert_eq!(*r, [6, 8]);

            let r = query(idx.eq("Paul")).or(idx.eq("Blub")).exec();
            assert_eq!(*r, [6]);

            let r = query(idx.eq("Blub")).or(idx.eq("Mario")).exec();
            assert_eq!(*r, [8]);
        }

        #[test]
        fn double_index() {
            let mut i = UniqueStrIdx::default();
            i.insert("Jasmin", 2).unwrap();

            assert_eq!(Err(Error::NotUniqueIndexKey), i.insert("Jasmin", 2));
        }

        #[test]
        fn out_of_bound() {
            let i = UniqueStrIdx::default();
            assert_eq!(0, i.eq("Jasmin").len());
        }
    }

    mod multi {
        use super::*;

        #[test]
        fn empty() {
            let i = MultiStrIdx::default();
            assert_eq!(0, i.eq("Jasmin").len());
            assert!(i.0.is_empty());
        }

        #[test]
        fn find_idx_2() {
            let mut i = MultiStrIdx::default();
            i.insert("Jasmin", 2).unwrap();

            assert_eq!(*i.eq("Jasmin"), [2]);
            assert_eq!(1, i.0.len());
        }

        #[test]
        fn double_index() {
            let mut i = MultiStrIdx::default();
            i.insert("Jasmin", 2).unwrap();
            i.insert("Jasmin", 1).unwrap();

            assert_eq!(*i.eq("Jasmin"), [1, 2]);
        }
    }
}
