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
    index::{Equals, Index, Store},
    Idx, EMPTY_IDXS,
};
use std::{borrow::Cow, collections::HashMap};

/// `Key` is from type [`str`] and use [`std::collections::BTreeMap`] for the searching.
#[derive(Debug, Default)]
pub struct StrMapIndex(HashMap<String, Index>);

impl Store<String> for StrMapIndex {
    fn insert(&mut self, key: String, i: Idx) {
        match self.0.get_mut(&key) {
            Some(v) => v.add(i),
            None => {
                self.0.insert(key, Index::new(i));
            }
        }
    }

    fn with_capacity(capacity: usize) -> Self {
        StrMapIndex(HashMap::with_capacity(capacity))
    }
}

impl<'k> Equals<&'k str> for StrMapIndex {
    #[inline]
    fn eq(&self, key: &'k str) -> Cow<[Idx]> {
        match self.0.get(key) {
            Some(i) => i.get(),
            None => Cow::Borrowed(EMPTY_IDXS),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::query;

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
            i.insert("Jasmin".into(), 4);

            assert_eq!(*i.eq("Jasmin"), [4]);
            assert_eq!(1, i.0.len());
        }

        #[test]
        fn or_find_idx_3_4() {
            let mut idx = StrMapIndex::default();
            idx.insert("Jasmin".into(), 4);
            idx.insert("Mario".into(), 8);
            idx.insert("Paul".into(), 6);

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

        #[test]
        fn find_eq_many_unique() {
            let mut idx = StrMapIndex::default();
            idx.insert("Jasmin".into(), 5);
            idx.insert("Mario".into(), 2);
            idx.insert("Paul".into(), 6);

            assert_eq!(0, idx.eq_iter([]).iter().len());
            assert_eq!(0, idx.eq_iter(["NotFound"]).iter().len());
            assert_eq!([2], *idx.eq_iter(["Mario"]));
            assert_eq!([2, 6], *idx.eq_iter(["Paul", "Mario"]));
            assert_eq!([2, 6], *idx.eq_iter(["NotFound", "Paul", "Mario"]));
            assert_eq!(
                [2, 5, 6],
                *idx.eq_iter(["Jasmin", "NotFound", "Mario", "Paul"])
            );
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
            i.insert("Jasmin".into(), 2);

            assert_eq!(*i.eq("Jasmin"), [2]);
            assert_eq!(1, i.0.len());
        }

        #[test]
        fn double_index() {
            let mut i = StrMapIndex::default();
            i.insert("Jasmin".into(), 2);
            i.insert("Jasmin".into(), 1);

            assert_eq!(*i.eq("Jasmin"), [1, 2]);
        }
    }
}
