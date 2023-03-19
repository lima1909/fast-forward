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
    query::EMPTY_IDXS,
    Idx, Key, Result,
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

impl<'s, I: Index> Store<'s> for StrMapIndex<'s, I> {
    fn insert(&mut self, k: Key<'s>, i: Idx) -> super::Result {
        match self.0.entry(k.try_into()?) {
            Entry::Vacant(e) => {
                e.insert(I::new(i));
                Ok(())
            }
            Entry::Occupied(mut e) => e.get_mut().add(i),
        }
    }
}

impl<'k, 's, I: Index> Filterable<'k> for StrMapIndex<'s, I> {
    fn filter(&self, p: Predicate<'k>) -> Result<Cow<[usize]>> {
        let s: &str = p.2.try_into()?;

        let idxs = match self.0.get(s) {
            Some(i) => Cow::Borrowed(i.get()),
            None => Cow::Borrowed(EMPTY_IDXS),
        };

        Ok(idxs)
    }
}

impl<'s, I: Index> StrMapIndex<'s, I> {
    pub fn insert_str(&mut self, k: &'s str, i: Idx) -> super::Result {
        self.insert(k.into(), i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::Queryable;
    use crate::{error::Error, index::OpsFilter};

    mod unique {
        use super::*;

        #[test]
        fn empty() {
            let i = UniqueStrIdx::default();
            assert_eq!(0, i.eq("Jasmin").unwrap().len());
            assert!(i.0.is_empty());
        }

        #[test]
        fn find_idx_2() {
            let mut i = UniqueStrIdx::default();
            i.insert_str("Jasmin", 4).unwrap();

            assert_eq!(*i.eq("Jasmin").unwrap(), [4]);
            assert_eq!(1, i.0.len());
        }

        #[test]
        fn or_find_idx_3_4() -> Result {
            let mut idx = UniqueStrIdx::default();
            idx.insert_str("Jasmin", 4).unwrap();
            idx.insert_str("Mario", 8).unwrap();
            idx.insert_str("Paul", 6).unwrap();

            let r = idx.query("Mario").or("Paul").exec()?;
            assert_eq!(*r, [6, 8]);

            let r = idx.query("Paul").or("Blub").exec()?;
            assert_eq!(*r, [6]);

            let r = idx.query("Blub").or("Mario").exec()?;
            assert_eq!(*r, [8]);

            Ok(())
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
            assert_eq!(0, i.eq("Jasmin").unwrap().len());
        }

        #[test]
        fn insert_invalid_key_type() {
            let mut i = UniqueStrIdx::default();
            let err = i.insert(Key::Usize(42), 4);
            assert!(err.is_err());
        }
    }

    mod multi {
        use super::*;

        #[test]
        fn empty() {
            let i = MultiStrIdx::default();
            assert_eq!(0, i.eq("Jasmin").unwrap().len());
            assert!(i.0.is_empty());
        }

        #[test]
        fn find_idx_2() {
            let mut i = MultiStrIdx::default();
            i.insert_str("Jasmin", 2).unwrap();

            assert_eq!(*i.eq("Jasmin").unwrap(), [2]);
            assert_eq!(1, i.0.len());
        }

        #[test]
        fn double_index() {
            let mut i = MultiStrIdx::default();
            i.insert_str("Jasmin", 2).unwrap();
            i.insert_str("Jasmin", 1).unwrap();

            assert_eq!(*i.eq("Jasmin").unwrap(), [1, 2]);
        }
    }
}
