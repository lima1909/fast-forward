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
use super::{Filter, IdxFilter, Index, KeyIdxStore, Multi, Unique};
use crate::{ops, Idx};
use std::{
    collections::{btree_map::Entry, BTreeMap},
    fmt::Debug,
};

/// Unique Key from type [`str`].
pub type UniqueStrIdx<'a> = StrMapIndex<'a, Unique>;

/// An not unique [`str`] Key, which can occur multiple times.
pub type MultiStrIdx<'a> = StrMapIndex<'a, Multi>;

/// `Key` is from type [`str`] and use [`std::collections::BTreeMap`] for the searching.
#[derive(Debug, Default)]
pub struct StrMapIndex<'a, I: Index>(BTreeMap<&'a str, I>);

impl<'a, I: Index> IdxFilter<&str> for StrMapIndex<'a, I> {
    fn idx(&self, f: Filter<&str>) -> &[Idx] {
        if f.op != ops::EQ {
            return &[];
        }

        match self.0.get(f.key) {
            Some(i) => i.get(),
            None => &[],
        }
    }
}

impl<'a, I: Index> KeyIdxStore<&'a str> for StrMapIndex<'a, I> {
    fn insert(&mut self, k: &'a str, i: Idx) -> super::Result {
        match self.0.entry(k) {
            Entry::Vacant(e) => {
                e.insert(I::new(i));
                Ok(())
            }
            Entry::Occupied(mut e) => e.get_mut().add(i),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod unique {
        use super::*;
        use std::collections::HashSet;

        use crate::{
            index::IndexError,
            query::{self, QueryBuilder, ToIdx},
        };

        impl<'a> ToIdx<'a> for UniqueStrIdx<'a> {
            fn to_idx(&self, f: crate::query::Filter<'a>) -> &[Idx] {
                self.idx(f.into())
            }
        }

        fn eq(v: &str) -> query::Filter<'_> {
            query::Filter::new("", crate::ops::EQ, query::Key::Str(v))
        }

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

            assert_eq!(i.eq("Jasmin"), &[4]);
            assert_eq!(1, i.0.len());
        }

        #[test]
        fn or_find_idx_3_4() {
            let mut idx = UniqueStrIdx::default();
            idx.insert("Jasmin", 4).unwrap();
            idx.insert("Mario", 8).unwrap();
            idx.insert("Paul", 6).unwrap();

            let b = QueryBuilder::<HashSet<Idx>, _>::new(idx);
            let r = b.query(eq("Mario")).or(eq("Paul")).exec();
            assert!(r.contains(&8));
            assert!(r.contains(&6));

            let r = b.query(eq("Paul")).or(eq("Blub")).exec();
            assert!(r.contains(&6));

            let r = b.query(eq("Blub")).or(eq("Mario")).exec();
            assert!(r.contains(&8));
        }

        #[test]
        fn double_index() {
            let mut i = UniqueStrIdx::default();
            i.insert("Jasmin", 2).unwrap();

            assert_eq!(Err(IndexError::NotUniqueKey), i.insert("Jasmin", 2));
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

            assert_eq!(i.eq("Jasmin"), &[2]);
            assert_eq!(1, i.0.len());
        }

        #[test]
        fn double_index() {
            let mut i = MultiStrIdx::default();
            i.insert("Jasmin", 2).unwrap();
            i.insert("Jasmin", 1).unwrap();

            assert_eq!(i.eq("Jasmin"), &[2, 1]);
        }
    }
}
