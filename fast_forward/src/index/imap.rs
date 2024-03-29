//! Is an `Index` which use tha hashing from the [`std::collections::HashMap`]
//! to find the Indices for a given `Key`.
//!
use crate::index::{
    indices::{KeyIndex, MultiKeyIndex},
    store::{Filterable, Store, View, ViewCreator},
};
use std::{fmt::Debug, hash::Hash};

#[cfg(feature = "hashbrown")]
use hashbrown::HashMap;

#[cfg(not(feature = "hashbrown"))]
use std::collections::HashMap;

/// `Key` default type is [`String`] and use [`std::collections::HashMap`] for the Index implementation.
#[derive(Debug)]
#[repr(transparent)]
pub struct MapIndex<K = String, X = usize>(HashMap<K, MultiKeyIndex<X>>);

impl<K, X> Default for MapIndex<K, X> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<K, X> Filterable for MapIndex<K, X>
where
    K: Hash + Eq,
    X: Ord + PartialEq,
{
    type Key = K;
    type Index = X;

    #[inline]
    fn get(&self, key: &Self::Key) -> &[Self::Index] {
        match self.0.get(key) {
            Some(i) => i.as_slice(),
            None => &[],
        }
    }

    fn contains(&self, key: &Self::Key) -> bool {
        self.0.contains_key(key)
    }
}

impl<'a, K, X> ViewCreator<'a> for MapIndex<K, X>
where
    K: Hash + Eq,
    X: Ord + 'a,
{
    type Key = K;
    type Filter = HashMap<K, &'a MultiKeyIndex<X>>;

    fn create_view<It>(&'a self, keys: It) -> View<Self::Filter>
    where
        It: IntoIterator<Item = Self::Key>,
    {
        let mut map = HashMap::<K, &MultiKeyIndex<X>>::with_capacity(self.0.len());

        for key in keys {
            if let Some(idxs) = self.0.get(&key).as_ref() {
                map.insert(key, *idxs);
            }
        }

        View(map)
    }
}

impl<K, X> Store for MapIndex<K, X>
where
    K: Hash + Eq,
    X: Ord,
{
    fn insert(&mut self, key: K, i: Self::Index) {
        match self.0.get_mut(&key) {
            Some(v) => v.add(i),
            None => {
                self.0.insert(key, MultiKeyIndex::new(i));
            }
        }
    }

    fn delete(&mut self, key: K, idx: &Self::Index) {
        if let Some(rm_idx) = self.0.get_mut(&key) {
            if rm_idx.remove(idx) {
                self.0.remove(&key);
            }
        }
    }

    fn with_capacity(capacity: usize) -> Self {
        MapIndex(HashMap::with_capacity(capacity))
    }
}

impl<K, X> Filterable for HashMap<K, &MultiKeyIndex<X>>
where
    K: Hash + Eq,
    X: Ord + PartialEq,
{
    type Key = K;
    type Index = X;

    #[inline]
    fn get(&self, key: &Self::Key) -> &[Self::Index] {
        match self.get(key) {
            Some(i) => i.as_slice(),
            None => &[],
        }
    }

    fn contains(&self, key: &Self::Key) -> bool {
        self.contains_key(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl<K: Default> MapIndex<K> {
        fn new() -> Self {
            Self(HashMap::new())
        }
    }

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

    #[test]
    fn index_str() {
        let mut i = MapIndex::<String, String>::default();
        i.insert("Jasmin".into(), "Jasmin".into());
        i.insert("Mario".into(), "Mario 1".into());
        i.insert("Mario".into(), "Mario 2".into());
        i.insert("Paul".into(), "Paul".into());

        assert!(i.contains(&"Paul".into()));

        for idx in i.get(&"Jasmin".into()).iter() {
            assert_eq!(&String::from("Jasmin"), idx);
        }

        let idxs = i.get(&"Jasmin".into());
        let mut it = idxs.iter();
        assert_eq!(Some(&"Jasmin".into()), it.next());
        assert_eq!(None, it.next());

        let idxs = i.get(&"Mario".into());
        let mut it = idxs.iter();
        assert_eq!(Some(&"Mario 1".into()), it.next());
        assert_eq!(Some(&"Mario 2".into()), it.next());
        assert_eq!(None, it.next());
    }

    mod unique {
        use super::{super::super::filter::Filter, *};

        #[test]
        fn empty() {
            let i = MapIndex::new();
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
            let i = MapIndex::new();
            assert_eq!(0, i.get(&"Jasmin").len());
        }

        #[test]
        fn find_eq_many_unique() {
            let l = [
                String::from("Jasmin"),
                String::from("Mario"),
                String::from("Paul"),
            ];
            let idx = MapIndex::from_list(l.clone());

            assert!(idx.get_many([]).items(&l).next().is_none());

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
            idx.delete("Jasmin", &100);
            assert_eq!([3, 4], idx.get(&"Jasmin"));

            // delete correct Key with correct Index
            idx.delete("Jasmin", &3);
            assert_eq!([4], idx.get(&"Jasmin"));

            // delete correct Key with last correct Index, Key now longer exist
            idx.delete("Jasmin", &4);
            assert!(idx.get(&"Jasmin").is_empty());

            // delete not exist Key
            idx.delete("NotExist", &1);
        }
    }

    mod multi {
        use super::*;

        #[test]
        fn empty() {
            let i = MapIndex::new();
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

        #[test]
        fn create_view() {
            let mut i = MapIndex::default();
            i.insert("Jasmin", 5);
            i.insert("Jasmin", 2);
            i.insert("Mario", 3);
            i.insert("Paul", 4);

            {
                let view = i.create_view(["Jasmin", "Mario"]);
                assert!(view.contains(&"Jasmin"));
                assert!(view.contains(&"Mario"));
                assert!(!view.contains(&"Paul"));

                assert_eq!(view.get(&"Jasmin"), &[2, 5]);
                assert_eq!(view.get(&"Mario"), &[3]);
                assert_eq!(view.get(&"Paul"), &[]);

                assert_eq!(
                    view.get_many(["Jasmin", "Mario"]).collect::<Vec<_>>(),
                    vec![&2, &5, &3]
                );
            }

            i.update("Paul", 4, "NEW");

            let view = i.create_view(["Jasmin", "NEW"]);
            assert_eq!(view.get(&"NEW"), &[4]);
            assert_eq!(view.get(&"Jasmin"), &[2, 5]);
        }
    }
}
