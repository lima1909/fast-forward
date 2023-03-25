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
    index::{Index, Store},
    query::EMPTY_IDXS,
    Idx,
};
use std::{borrow::Cow, collections::HashMap, fmt::Debug};

/// `Key` is from type [`str`] and use [`std::collections::BTreeMap`] for the searching.
#[derive(Debug, Default)]
pub struct StrMapIndex<'s>(HashMap<&'s str, Index>);

impl<'s> Store<&'s str> for StrMapIndex<'s> {
    fn insert(&mut self, key: &'s str, i: Idx) {
        match self.0.get_mut(key) {
            Some(v) => v.add(i),
            None => {
                self.0.insert(key, Index::new(i));
            }
        }
    }
}

impl<'s> StrMapIndex<'s> {
    pub fn eq(&self, key: &'s str) -> Cow<[Idx]> {
        match self.0.get(key) {
            Some(i) => i.get(),
            None => Cow::Borrowed(EMPTY_IDXS),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query;

    mod unique {
        use super::*;

        #[test]
        fn empty() {
            let i = StrMapIndex::default();
            assert_eq!(0, i.eq("Jasmin").len());
            assert!(i.0.is_empty());
        }

        #[test]
        fn find_idx_2() {
            let mut i = StrMapIndex::default();
            i.insert("Jasmin", 4);

            assert_eq!(*i.eq("Jasmin"), [4]);
            assert_eq!(1, i.0.len());
        }

        #[test]
        fn or_find_idx_3_4() {
            let mut idx = StrMapIndex::default();
            idx.insert("Jasmin", 4);
            idx.insert("Mario", 8);
            idx.insert("Paul", 6);

            let r = query(idx.eq("Mario")).or(idx.eq("Paul")).exec();
            assert_eq!(*r, [6, 8]);

            let r = query(idx.eq("Paul")).or(idx.eq("Blub")).exec();
            assert_eq!(*r, [6]);

            let r = query(idx.eq("Blub")).or(idx.eq("Mario")).exec();
            assert_eq!(*r, [8]);
        }

        #[test]
        fn out_of_bound() {
            let i = StrMapIndex::default();
            assert_eq!(0, i.eq("Jasmin").len());
        }
    }

    mod multi {
        use super::*;

        #[test]
        fn empty() {
            let i = StrMapIndex::default();
            assert_eq!(0, i.eq("Jasmin").len());
            assert!(i.0.is_empty());
        }

        #[test]
        fn find_idx_2() {
            let mut i = StrMapIndex::default();
            i.insert("Jasmin", 2);

            assert_eq!(*i.eq("Jasmin"), [2]);
            assert_eq!(1, i.0.len());
        }

        #[test]
        fn double_index() {
            let mut i = StrMapIndex::default();
            i.insert("Jasmin", 2);
            i.insert("Jasmin", 1);

            assert_eq!(*i.eq("Jasmin"), [1, 2]);
        }
    }
}
