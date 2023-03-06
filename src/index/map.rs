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
    index::{Filterable, Index, Multi, Predicate, Store, Unique},
    Idx, Key,
};
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

impl<'k, I: Index> Store<'k> for StrMapIndex<'k, I> {
    fn insert(&mut self, k: Key<'k>, i: Idx) -> super::Result {
        match self.0.entry(k.into()) {
            Entry::Vacant(e) => {
                e.insert(I::new(i));
                Ok(())
            }
            Entry::Occupied(mut e) => e.get_mut().add(i),
        }
    }
}

impl<'k, I: Index> Filterable<'k> for StrMapIndex<'k, I> {
    fn filter(&self, p: Predicate<'k>) -> &[Idx] {
        match self.0.get(p.2.into()) {
            Some(i) => i.get(),
            None => &[],
        }
    }
}

impl<'k, I: Index> StrMapIndex<'k, I> {
    pub fn insert_str(&mut self, k: &'k str, i: Idx) -> super::Result {
        self.insert(k.into(), i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::Queryable;
    use crate::{error::Error, index::OpsFilter};
    use std::collections::HashSet;

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
            i.insert_str("Jasmin", 4).unwrap();

            assert_eq!(i.eq("Jasmin"), &[4]);
            assert_eq!(1, i.0.len());
        }

        #[test]
        fn or_find_idx_3_4() {
            let mut idx = UniqueStrIdx::default();
            idx.insert_str("Jasmin", 4).unwrap();
            idx.insert_str("Mario", 8).unwrap();
            idx.insert_str("Paul", 6).unwrap();

            let b = idx.query_builder::<HashSet<Idx>>();
            let r: Vec<Idx> = b.query("Mario").or("Paul").exec().collect();
            assert!(r.contains(&8));
            assert!(r.contains(&6));

            let mut r = b.query("Paul").or("Blub").exec();
            assert_eq!(r.next(), Some(6));

            let mut r = b.query("Blub").or("Mario").exec();
            assert_eq!(r.next(), Some(8));
        }

        #[test]
        fn double_index() {
            let mut i = UniqueStrIdx::default();
            i.insert_str("Jasmin", 2).unwrap();

            assert_eq!(Err(Error::NotUniqueIndexKey), i.insert_str("Jasmin", 2));
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
            i.insert_str("Jasmin", 2).unwrap();

            assert_eq!(i.eq("Jasmin"), &[2]);
            assert_eq!(1, i.0.len());
        }

        #[test]
        fn double_index() {
            let mut i = MultiStrIdx::default();
            i.insert_str("Jasmin", 2).unwrap();
            i.insert_str("Jasmin", 1).unwrap();

            assert_eq!(i.eq("Jasmin"), &[2, 1]);
        }
    }
}
