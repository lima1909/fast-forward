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
    index::{EqFilter, Index, ItemRetriever, NoMeta, Retriever, Store},
    ListIndexFilter, SelIdx,
};
use std::{collections::HashMap, fmt::Debug, hash::Hash};

/// `Key` is from type [`str`] and use [`std::collections::BTreeMap`] for the searching.
#[derive(Debug, Default)]
pub struct MapIndex<K: Default = String>(HashMap<K, Index>);

impl<K> Store for MapIndex<K>
where
    K: Default + Eq + Hash,
{
    type Key = K;

    fn insert(&mut self, key: K, i: usize) {
        match self.0.get_mut(&key) {
            Some(v) => v.add(i),
            None => {
                self.0.insert(key, Index::new(i));
            }
        }
    }

    fn delete(&mut self, key: K, idx: usize) {
        if let Some(rm_idx) = self.0.get_mut(&key) {
            if rm_idx.remove(idx).is_empty() {
                self.0.remove(&key);
            }
        }
    }

    fn with_capacity(capacity: usize) -> Self {
        MapIndex(HashMap::with_capacity(capacity))
    }

    type Retriever<'a> = MapIndex<K> where K:'a;

    fn retrieve<'a, I, L>(&'a self, items: &'a L) -> ItemRetriever<'a, Self::Retriever<'a>, L>
    where
        I: 'a,
        L: ListIndexFilter<Item = I> + 'a,
    {
        ItemRetriever { inner: self, items }
    }
}

impl<K> Retriever for MapIndex<K>
where
    K: Default + Eq + Hash,
{
    type Key = K;

    fn get(&self, key: &Self::Key) -> SelIdx<'_> {
        match self.0.get(key) {
            Some(i) => i.get(),
            None => SelIdx::empty(),
        }
    }

    type Meta<'f> = NoMeta where K:'f;

    fn meta(&self) -> Self::Meta<'_> {
        NoMeta
    }

    type Filter<'f> = EqFilter<'f, Self> where K:'f;

    fn filter<'s, P>(&'s self, predicate: P) -> SelIdx<'_>
    where
        P: Fn(<Self as Retriever>::Filter<'s>) -> SelIdx<'_>,
    {
        predicate(EqFilter(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retrieve() {
        let mut i = MapIndex::default();
        i.insert("Jasmin", 4);
        i.insert("Mario", 8);
        i.insert("Paul", 6);

        assert!(i.contains(&"Paul"));

        let items = vec!["a", "b", "c", "d", "e"];

        let r = i.retrieve(&items);
        let mut it = r.filter(|f| f.eq(&"Jasmin"));
        assert_eq!(Some(&"e"), it.next());
        assert_eq!(None, it.next());

        assert!(i.meta().has_no_meta_data());
    }

    mod unique {
        use super::*;

        #[test]
        fn empty() {
            let i = MapIndex::default();
            assert_eq!(0, i.get(&"Jasmin").len());
            assert!(i.0.is_empty());
        }

        #[test]
        fn find_idx_2_str() {
            let mut i = MapIndex::default();
            i.insert("Jasmin", 4);

            assert_eq!(i.get(&"Jasmin"), [4]);
            assert_eq!(1, i.0.len());
        }

        #[test]
        fn find_idx_2_i32() {
            let mut i = MapIndex::default();
            i.insert(5, 4);

            assert_eq!(i.get(&5), [4]);
            assert_eq!(1, i.0.len());
        }

        #[test]
        fn find_idx_2_char() {
            let mut i = MapIndex::default();
            i.insert('x', 4);

            assert_eq!(i.get(&'x'), [4]);
            assert_eq!(1, i.0.len());
        }

        #[test]
        fn or_find_idx_3_4() {
            let mut idx = MapIndex::default();
            idx.insert("Jasmin", 4);
            idx.insert("Mario", 8);
            idx.insert("Paul", 6);

            let r = idx.get(&"Mario") | idx.get(&"Paul");
            assert_eq!(r, [6, 8]);

            let r = idx.get(&"Paul") | idx.get(&"Blub");
            assert_eq!(r, [6]);

            let r = idx.get(&"Blub") | idx.get(&"Mario");
            assert_eq!(r, [8]);
        }

        #[test]
        fn out_of_bound() {
            let i = MapIndex::default();
            assert_eq!(0, i.get(&"Jasmin").len());
        }

        #[test]
        fn find_eq_many_unique() {
            let mut idx = MapIndex::default();
            idx.insert("Jasmin", 5);
            idx.insert("Mario", 2);
            idx.insert("Paul", 6);

            assert_eq!(0, idx.get_many([]).iter().len());
            assert_eq!(0, idx.get_many(["NotFound"]).iter().len());
            assert_eq!([2], idx.get_many(["Mario"]));
            assert_eq!([2, 6], idx.get_many(["Paul", "Mario"]));
            assert_eq!([2, 6], idx.get_many(["NotFound", "Paul", "Mario"]));
            assert_eq!(
                [2, 5, 6],
                idx.get_many(["Jasmin", "NotFound", "Mario", "Paul"])
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

        #[test]
        fn update() {
            let mut idx = MapIndex::default();
            idx.insert("Jasmin", 5);

            // (old) Key: Jasmin do not exist, insert a (new) Key Jasmin NEW?
            idx.update("Jasmin", 4, "Jasmin NEW");
            assert_eq!([4], idx.get(&"Jasmin NEW"));

            // (old) Key 2 exist, but not with Index: 8, insert known Key: 2 with add new Index 8
            idx.update("Jasmin NEW", 8, "Jasmin NEW");
            assert_eq!([4, 8], idx.get(&"Jasmin NEW"));

            // old Key 2 with Index 8 was removed and (new) Key 4 was added with Index 8
            idx.update("Jasmin NEW", 8, "Jasmin NEW NEW");
            assert_eq!([8], idx.get(&"Jasmin NEW NEW"));
            assert_eq!([4], idx.get(&"Jasmin NEW"));
        }

        #[test]
        fn delete() {
            let mut idx = MapIndex::default();
            idx.insert("Jasmin", 4);
            idx.insert("Jasmin", 3);
            idx.insert("Mario", 1);

            // delete correct Key with wrong Index, nothing happens
            idx.delete("Jasmin", 100);
            assert_eq!([3, 4], idx.get(&"Jasmin"));

            // delete correct Key with correct Index
            idx.delete("Jasmin", 3);
            assert_eq!([4], idx.get(&"Jasmin"));

            // delete correct Key with last correct Index, Key now longer exist
            idx.delete("Jasmin", 4);
            assert!(idx.get(&"Jasmin").is_empty());
        }
    }

    mod multi {
        use super::*;

        #[test]
        fn empty() {
            let i = MapIndex::default();
            assert_eq!(0, i.get(&"Jasmin").len());
            assert!(i.0.is_empty());
        }

        #[test]
        fn find_idx_2() {
            let mut i = MapIndex::default();
            i.insert("Jasmin", 2);

            assert_eq!(i.get(&"Jasmin"), [2]);
            assert_eq!(1, i.0.len());
        }

        #[test]
        fn double_index() {
            let mut i = MapIndex::default();
            i.insert("Jasmin", 2);
            i.insert("Jasmin", 1);

            assert_eq!(i.get(&"Jasmin"), [1, 2]);
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
