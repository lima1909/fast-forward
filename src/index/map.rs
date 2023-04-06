//! Indices for `Key`s  which implement: [`std::hash::Hash`] + [`std::cmp::Eq`].
//!
//! The `Key` is the Hash-Key and the value are the `Index` which are saved in the [`MapIndex`]:
//!
//!
//!```text
//! let _list_names_unique = vec!["Paul", "Mario", "Jasmin", ...];
//!
//! Unique [`MapIndex`]:
//!
//!  Key      | Idx
//! --------------------
//!  "Jasmin" |  2
//!  "Mario"  |  1
//!  "Paul"   |  0
//!   ...     | ...
//!
//! let _list_names__multi = vec!["Jasmin", "Mario", "Jasmin", ...];
//!
//! Multi [`MapIndex`]:
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
use std::{borrow::Cow, collections::HashMap, fmt::Debug, hash::Hash};

/// `Key` is from type [`str`] and use [`std::collections::BTreeMap`] for the searching.
#[derive(Debug, Default)]
pub struct MapIndex<K: Default = String>(HashMap<K, Index>);

impl<K> Store<K> for MapIndex<K>
where
    K: Default + Eq + Hash,
{
    fn insert(&mut self, key: K, i: Idx) {
        match self.0.get_mut(&key) {
            Some(v) => v.add(i),
            None => {
                self.0.insert(key, Index::new(i));
            }
        }
    }

    fn with_capacity(capacity: usize) -> Self {
        MapIndex(HashMap::with_capacity(capacity))
    }
}

impl<K> Equals<&K> for MapIndex<K>
where
    K: Default + Eq + Hash,
{
    #[inline]
    fn eq(&self, key: &K) -> Cow<[Idx]> {
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
            let i = MapIndex::default();
            assert_eq!(0, i.eq(&"Jasmin").len());
            assert!(i.0.is_empty());
        }

        #[test]
        fn find_idx_2_str() {
            let mut i = MapIndex::default();
            i.insert("Jasmin", 4);

            assert_eq!(*i.eq(&"Jasmin"), [4]);
            assert_eq!(1, i.0.len());
        }

        #[test]
        fn find_idx_2_i32() {
            let mut i = MapIndex::default();
            i.insert(5, 4);

            assert_eq!(*i.eq(&5), [4]);
            assert_eq!(1, i.0.len());
        }

        #[test]
        fn find_idx_2_char() {
            let mut i = MapIndex::default();
            i.insert('x', 4);

            assert_eq!(*i.eq(&'x'), [4]);
            assert_eq!(1, i.0.len());
        }

        #[test]
        fn or_find_idx_3_4() {
            let mut idx = MapIndex::default();
            idx.insert("Jasmin", 4);
            idx.insert("Mario", 8);
            idx.insert("Paul", 6);

            let r = query(idx.eq(&"Mario")).or(idx.eq(&"Paul")).exec();
            assert_eq!(*r, [6, 8]);

            let r = query(idx.eq(&"Paul")).or(idx.eq(&"Blub")).exec();
            assert_eq!(*r, [6]);

            let r = query(idx.eq(&"Blub")).or(idx.eq(&"Mario")).exec();
            assert_eq!(*r, [8]);
        }

        #[test]
        fn out_of_bound() {
            let i = MapIndex::default();
            assert_eq!(0, i.eq(&"Jasmin").len());
        }

        #[test]
        fn find_eq_many_unique() {
            let mut idx = MapIndex::default();
            idx.insert("Jasmin", 5);
            idx.insert("Mario", 2);
            idx.insert("Paul", 6);

            assert_eq!(0, idx.eq_iter([]).iter().len());
            assert_eq!(0, idx.eq_iter([&"NotFound"]).iter().len());
            assert_eq!([2], *idx.eq_iter([&"Mario"]));
            assert_eq!([2, 6], *idx.eq_iter([&"Paul", &"Mario"]));
            assert_eq!([2, 6], *idx.eq_iter([&"NotFound", &"Paul", &"Mario"]));
            assert_eq!(
                [2, 5, 6],
                *idx.eq_iter([&"Jasmin", &"NotFound", &"Mario", &"Paul"])
            );
        }

        #[test]
        fn contains() {
            let mut idx = MapIndex::default();
            idx.insert("Jasmin", 5);
            idx.insert("Mario", 2);

            assert!(idx.contains(&"Jasmin"));
            assert!(!idx.contains(&"Paul"));
        }
    }

    mod multi {
        use super::*;

        #[test]
        fn empty() {
            let i = MapIndex::default();
            assert_eq!(0, i.eq(&"Jasmin").len());
            assert!(i.0.is_empty());
        }

        #[test]
        fn find_idx_2() {
            let mut i = MapIndex::default();
            i.insert("Jasmin", 2);

            assert_eq!(*i.eq(&"Jasmin"), [2]);
            assert_eq!(1, i.0.len());
        }

        #[test]
        fn double_index() {
            let mut i = MapIndex::default();
            i.insert("Jasmin", 2);
            i.insert("Jasmin", 1);

            assert_eq!(*i.eq(&"Jasmin"), [1, 2]);
        }

        #[test]
        fn contains() {
            let mut idx = MapIndex::default();
            idx.insert("Jasmin", 5);
            idx.insert("Jasmin", 2);

            assert!(idx.contains(&"Jasmin"));
            assert!(!idx.contains(&"Paul"));
        }
    }
}
