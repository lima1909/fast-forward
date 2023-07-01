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
use crate::index::{indices::EMPTY_INDICES, store::Filterable, KeyIndices, Store};
use std::{collections::HashMap, fmt::Debug, hash::Hash};

/// `Key` is from type [`str`] and use [`std::collections::BTreeMap`] for the searching.
#[derive(Debug, Default)]
#[repr(transparent)]
pub struct MapIndex<K: Default = String>(HashMap<K, KeyIndices>);

impl<K> Filterable for MapIndex<K>
where
    K: Default + Hash + Eq,
{
    type Key = K;

    #[inline]
    fn get(&self, key: &Self::Key) -> &[usize] {
        match self.0.get(key) {
            Some(i) => i.as_slice(),
            None => EMPTY_INDICES,
        }
    }

    fn contains(&self, key: &Self::Key) -> bool {
        self.0.contains_key(key)
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
                self.0.insert(key, KeyIndices::new(i));
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

        for idx in i.get(&"Jasmin").iter() {
            assert_eq!(&4, idx);
        }

        let idxs = i.get(&"Jasmin");
        let mut it = idxs.iter();
        assert_eq!(Some(&4), it.next());
        assert_eq!(None, it.next());
    }

    mod unique {
        use super::*;
        use crate::index::store::Filter;

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

            let f = Filter(&idx);

            let r = f.eq(&"Mario") | f.eq(&"Paul");
            assert_eq!([6, 8], r);

            let r = f.eq(&"Paul") | f.eq(&"Blub");
            assert_eq!([6], r);

            let r = f.eq(&"Blub") | f.eq(&"Mario");
            assert_eq!([8], r);
        }

        #[test]
        fn out_of_bound() {
            let i = MapIndex::default();
            assert_eq!(0, i.get(&"Jasmin").len());
        }

        #[test]
        fn find_eq_many_unique() {
            let l = [
                String::from("Jasmin"),
                String::from("Mario"),
                String::from("Paul"),
            ];
            let idx = MapIndex::from_iter(l.clone().into_iter());

            assert_eq!(0, idx.get_many([]).items_vec(&l).len());
            assert_eq!(0, idx.get_many(["NotFound".into()]).items_vec(&l).len());
            assert_eq!(
                vec![&String::from("Mario")],
                idx.get_many(["Mario".into()]).items_vec(&l)
            );
            assert_eq!(
                vec![&String::from("Paul"), &String::from("Mario")],
                idx.get_many(["Paul".into(), "Mario".into()]).items_vec(&l)
            );
            assert_eq!(
                vec![&String::from("Paul"), &String::from("Mario")],
                idx.get_many(["NotFound".into(), "Paul".into(), "Mario".into()])
                    .items_vec(&l)
            );
            assert_eq!(
                vec![
                    &String::from("Jasmin"),
                    &String::from("Mario"),
                    &String::from("Paul")
                ],
                idx.get_many([
                    "Jasmin".into(),
                    "NotFound".into(),
                    "Mario".into(),
                    "Paul".into()
                ],)
                    .items_vec(&l)
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
