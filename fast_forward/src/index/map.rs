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
use crate::index::{store::Filterable, Indices, SelectedIndices, Store};
use std::{collections::HashMap, fmt::Debug, hash::Hash};

/// `Key` is from type [`str`] and use [`std::collections::BTreeMap`] for the searching.
#[derive(Debug, Default)]
pub struct MapIndex<K: Default = String>(HashMap<K, Indices>);

impl<K> Filterable for MapIndex<K>
where
    K: Default + Hash + Eq,
{
    type Key = K;

    #[inline]
    fn indices(&self, key: &Self::Key) -> SelectedIndices<'_> {
        match self.0.get(key) {
            Some(i) => i.get(),
            None => SelectedIndices::empty(),
        }
    }
}

impl<K> Store for MapIndex<K>
where
    K: Default + Eq + Hash,
{
    fn insert(&mut self, key: K, i: usize) {
        match self.0.get_mut(&key) {
            Some(v) => v.add(i),
            None => {
                self.0.insert(key, Indices::new(i));
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
}

// impl<K> Retriever for MapIndex<K>
// where
//     K: Default + Eq + Hash,
// {
//     type Key = K;

//     fn get(&self, key: &Self::Key) -> SelectedIndices<'_> {
//         match self.0.get(key) {
//             Some(i) => i.get(),
//             None => SelectedIndices::empty(),
//         }
//     }

//     type Meta<'f> = NoMeta where K:'f;

//     fn meta(&self) -> Self::Meta<'_> {
//         NoMeta
//     }

//     type Filter<'f> = EqFilter<'f, Self> where K:'f;

//     fn filter<'s, P>(&'s self, predicate: P) -> SelectedIndices<'_>
//     where
//         P: Fn(<Self as Retriever>::Filter<'s>) -> SelectedIndices<'_>,
//     {
//         predicate(EqFilter::new(self))
//     }
// }

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

        // TODO lifetime Iter
        let idxs = i.retrieve().filter(|f| f.eq(&"Jasmin"));
        let mut it = idxs.iter();
        assert_eq!(Some(&4), it.next());
        assert_eq!(None, it.next());
    }

    mod unique {
        use super::*;

        #[test]
        fn empty() {
            let i = MapIndex::default();
            assert_eq!(0, i.indices(&"Jasmin").len());
            assert!(i.0.is_empty());
        }

        #[test]
        fn find_idx_2_str() {
            let mut i = MapIndex::default();
            i.insert("Jasmin", 4);

            assert_eq!(i.indices(&"Jasmin"), [4]);
            assert_eq!(1, i.0.len());
        }

        #[test]
        fn find_idx_2_i32() {
            let mut i = MapIndex::default();
            i.insert(5, 4);

            assert_eq!(i.indices(&5), [4]);
            assert_eq!(1, i.0.len());
        }

        #[test]
        fn find_idx_2_char() {
            let mut i = MapIndex::default();
            i.insert('x', 4);

            assert_eq!(i.indices(&'x'), [4]);
            assert_eq!(1, i.0.len());
        }

        #[test]
        fn or_find_idx_3_4() {
            let mut idx = MapIndex::default();
            idx.insert("Jasmin", 4);
            idx.insert("Mario", 8);
            idx.insert("Paul", 6);

            let r = idx.indices(&"Mario") | idx.indices(&"Paul");
            assert_eq!(r, [6, 8]);

            let r = idx.indices(&"Paul") | idx.indices(&"Blub");
            assert_eq!(r, [6]);

            let r = idx.indices(&"Blub") | idx.indices(&"Mario");
            assert_eq!(r, [8]);
        }

        #[test]
        fn out_of_bound() {
            let i = MapIndex::default();
            assert_eq!(0, i.indices(&"Jasmin").len());
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
            assert_eq!([4], idx.indices(&"Jasmin NEW"));

            // (old) Key 2 exist, but not with Index: 8, insert known Key: 2 with add new Index 8
            idx.update("Jasmin NEW", 8, "Jasmin NEW");
            assert_eq!([4, 8], idx.indices(&"Jasmin NEW"));

            // old Key 2 with Index 8 was removed and (new) Key 4 was added with Index 8
            idx.update("Jasmin NEW", 8, "Jasmin NEW NEW");
            assert_eq!([8], idx.indices(&"Jasmin NEW NEW"));
            assert_eq!([4], idx.indices(&"Jasmin NEW"));
        }

        #[test]
        fn delete() {
            let mut idx = MapIndex::default();
            idx.insert("Jasmin", 4);
            idx.insert("Jasmin", 3);
            idx.insert("Mario", 1);

            // delete correct Key with wrong Index, nothing happens
            idx.delete("Jasmin", 100);
            assert_eq!([3, 4], idx.indices(&"Jasmin"));

            // delete correct Key with correct Index
            idx.delete("Jasmin", 3);
            assert_eq!([4], idx.indices(&"Jasmin"));

            // delete correct Key with last correct Index, Key now longer exist
            idx.delete("Jasmin", 4);
            assert!(idx.indices(&"Jasmin").is_empty());
        }
    }

    mod multi {
        use super::*;

        #[test]
        fn empty() {
            let i = MapIndex::default();
            assert_eq!(0, i.indices(&"Jasmin").len());
            assert!(i.0.is_empty());
        }

        #[test]
        fn find_idx_2() {
            let mut i = MapIndex::default();
            i.insert("Jasmin", 2);

            assert_eq!(i.indices(&"Jasmin"), [2]);
            assert_eq!(1, i.0.len());
        }

        #[test]
        fn double_index() {
            let mut i = MapIndex::default();
            i.insert("Jasmin", 2);
            i.insert("Jasmin", 1);

            assert_eq!(i.indices(&"Jasmin"), [1, 2]);
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
