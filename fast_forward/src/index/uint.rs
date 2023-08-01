//! This `Index` is well suitable for `IDs` with [`usize`] compatible data types (for example `Primary Keys`).
//!
use crate::index::{
    indices::KeyIndices,
    ops::MinMax,
    store::{Filterable, MetaData, Store},
    view::Keys,
};
use std::marker::PhantomData;

/// `Key` is from type [`usize`] and the information are saved in a List (Store).
#[derive(Debug)]
pub struct UIntIndex<K = usize, X = usize> {
    data: Vec<Option<(K, KeyIndices<X>)>>,
    min_max_cache: MinMax<K>,
    _key: PhantomData<K>,
}

impl<K, X> Default for UIntIndex<K, X>
where
    K: Default,
{
    fn default() -> Self {
        Self {
            data: vec![],
            min_max_cache: MinMax::default(),
            _key: PhantomData,
        }
    }
}

impl<K, X> Filterable for UIntIndex<K, X>
where
    K: Into<usize> + Copy,
{
    type Key = K;
    type Index = X;

    #[inline]
    fn get(&self, key: &Self::Key) -> &[X] {
        let i: usize = (*key).into();
        match self.data.get(i) {
            Some(Some((_, idx))) => idx.as_slice(),
            _ => &[],
        }
    }

    #[inline]
    fn contains(&self, key: &Self::Key) -> bool {
        matches!(self.data.get((*key).into()), Some(Some(_)))
    }
}

impl<K, X> Store for UIntIndex<K, X>
where
    K: Default + Into<usize> + Copy + Ord,
    X: Ord + Clone,
{
    fn insert(&mut self, k: K, i: X) {
        let orig_key = k;
        let k = k.into();

        if self.data.len() <= k {
            self.data.resize(k + 1, None);
        }

        match self.data[k].as_mut() {
            Some((_, idx)) => idx.add(i),
            None => self.data[k] = Some((orig_key, KeyIndices::new(i))),
        }

        self.min_max_cache.new_min_value(orig_key);
        self.min_max_cache.new_max_value(orig_key);
    }

    fn delete(&mut self, key: K, idx: &X) {
        let orig_key = key;
        let k = key.into();
        if let Some(Some((_, rm_idx))) = self.data.get_mut(k) {
            // if the Index is the last, then remove complete Index
            if rm_idx.remove(idx).is_empty() {
                self.data[k] = None
            }
        }

        if orig_key == self.min_max_cache.min {
            self.min_max_cache.min = self._find_min();
        }
        if orig_key == self.min_max_cache.max {
            self.min_max_cache.max = self._find_max();
        }
    }

    fn with_capacity(capacity: usize) -> Self {
        UIntIndex {
            data: Vec::with_capacity(capacity),
            min_max_cache: MinMax::default(),
            _key: PhantomData,
        }
    }
}

impl<K> Keys for UIntIndex<K>
where
    K: Default + Into<usize> + Copy + Ord,
{
    type Key = K;

    fn exist(&self, key: &K) -> bool {
        matches!(self.data.get((*key).into()), Some(Some(_)))
    }

    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = &'a Self::Key> + 'a> {
        Box::new(self.data.iter().filter_map(|o| o.as_ref().map(|(k, _)| k)))
    }

    fn from_iter<I>(it: I) -> Self
    where
        I: IntoIterator<Item = K>,
    {
        fn add_key<K>(view: &mut UIntIndex<K>, key: K)
        where
            K: Into<usize> + Copy + Ord,
        {
            let orig_key = key;
            let pos: usize = key.into();

            if view.data.len() <= pos {
                view.data.resize(pos + 1, None);
            }

            view.data[pos] = Some((orig_key, KeyIndices::empty()))
        }

        let v = Vec::from_iter(it);
        let mut view = Self::with_capacity(v.iter().max().map(|k| (*k).into()).unwrap_or_default());
        v.into_iter().for_each(|key| add_key(&mut view, key));
        view
    }
}

impl<K, X> MetaData for UIntIndex<K, X> {
    type Meta<'m> = UIntMeta<'m, K,X> where K: 'm, X:'m;

    fn meta(&self) -> Self::Meta<'_> {
        UIntMeta(self)
    }
}

/// Meta data for the UIntIndex, like min and max value from the saved Index.
pub struct UIntMeta<'s, K: 's, X>(&'s UIntIndex<K, X>);

impl<'s, K, X> UIntMeta<'s, K, X>
where
    K: Copy + 's,
{
    /// Filter for get the smallest (`min`) `Key` which is stored in `UIntIndex`.
    pub const fn min_key(&self) -> K {
        self.0.min_max_cache.min
    }

    /// Filter for get the highest (`max`) `Key` which is stored in `UIntIndex`.
    pub const fn max_key(&self) -> K {
        self.0.min_max_cache.max
    }
}

impl<K, X> UIntIndex<K, X>
where
    K: Default + Copy,
{
    /// Find `min` key.
    fn _find_min(&self) -> K {
        self.data
            .iter()
            .find_map(|o| o.as_ref().map(|(k, _)| *k))
            .unwrap_or_default()
    }

    /// Find `max` key.
    fn _find_max(&self) -> K {
        self.data
            .iter()
            .rev()
            .find_map(|o| o.as_ref().map(|(k, _)| *k))
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::{super::filter::Filter, *};

    impl UIntIndex<usize> {
        fn new() -> Self {
            Self {
                data: Vec::new(),
                min_max_cache: MinMax::default(),
                _key: PhantomData,
            }
        }
    }

    #[test]
    fn retrieve() {
        let mut i = UIntIndex::new();
        i.insert(1, 3);
        i.insert(2, 4);

        let idxs = i.get(&2);
        let mut it = idxs.iter();
        assert_eq!(Some(&4), it.next());
        assert_eq!(None, it.next());

        assert_eq!(1, i.meta().min_key());
        assert_eq!(2, i.meta().max_key());
    }

    #[test]
    fn filter() {
        let mut i = UIntIndex::new();
        i.insert(2, 4);

        assert_eq!(i.get(&2), [4]);

        i.insert(1, 3);
        let f = Filter(&i);
        assert_eq!([3, 4], (f.eq(&2) | f.eq(&1)));
    }

    #[test]
    fn meta() {
        let mut i = UIntIndex::new();
        i.insert(2, 4);

        assert_eq!(2, i.meta().min_key());
        assert_eq!(2, i.meta().max_key());

        i.insert(1, 3);
        assert_eq!(1, i.meta().min_key());
        assert_eq!(2, i.meta().max_key());
    }

    #[test]
    fn index_str() {
        let mut i = UIntIndex::<usize, String>::default();
        i.insert(1, "Jasmin".into());
        i.insert(2, "Mario 1".into());
        i.insert(2, "Mario 2".into());
        i.insert(5, "Paul".into());

        assert!(i.contains(&5));

        for idx in i.get(&1).iter() {
            assert_eq!(&String::from("Jasmin"), idx);
        }

        let idxs = i.get(&1);
        let mut it = idxs.iter();
        assert_eq!(Some(&"Jasmin".into()), it.next());
        assert_eq!(None, it.next());

        let idxs = i.get(&2);
        let mut it = idxs.iter();
        assert_eq!(Some(&"Mario 1".into()), it.next());
        assert_eq!(Some(&"Mario 2".into()), it.next());
        assert_eq!(None, it.next());
    }

    mod unique {
        use super::*;

        #[test]
        fn empty() {
            let i = UIntIndex::new();
            assert_eq!(0, i.get(&2).len());
            assert!(i.data.is_empty());
        }

        #[test]
        fn find_idx_2_usize() {
            let mut i = UIntIndex::new();
            i.insert(2, 4);

            assert_eq!(i.get(&2), [4]);
            assert_eq!(3, i.data.len());
        }

        #[test]
        fn find_idx_2_bool() {
            let mut i = UIntIndex::<bool>::default();
            i.insert(true, 4);

            assert_eq!(i.get(&true), [4]);
            assert_eq!(2, i.data.len());
        }

        #[test]
        fn find_idx_2_u16() {
            let mut i = UIntIndex::<u16>::default();
            i.insert(2, 4);

            assert_eq!(i.get(&2), [4]);
            assert_eq!(3, i.data.len());
        }

        #[test]
        fn or_find_idx_3_4() {
            let mut idx = UIntIndex::new();
            idx.insert(2, 4);
            idx.insert(4, 8);
            idx.insert(3, 6);

            let f = Filter(&idx);

            assert_eq!([6, 8], f.eq(&3) | f.eq(&4));
            assert_eq!([6], f.eq(&3) & f.eq(&3));
            assert_eq!([6], f.eq(&3) | f.eq(&99));
            assert_eq!([8], f.eq(&99) | f.eq(&4));
            assert_eq!([], f.eq(&3) & f.eq(&4));

            idx.insert(99, 0);
            assert_eq!([0], idx.get(&99));
        }

        #[test]
        fn query_and_or() {
            let mut idx = UIntIndex::<usize>::default();
            idx.insert(2, 4);
            idx.insert(4, 8);
            idx.insert(3, 6);

            let f = Filter(&idx);

            assert_eq!([], f.eq(&3) & f.eq(&2));

            // =3 or =4 and =2 =>
            // (
            // (4 and 2 = false) // `and` has higher prio than `or`
            //  or 3 = true
            // )
            // => 3 -> 6
            assert_eq!([6], f.eq(&3) | f.eq(&4) & f.eq(&2));
        }

        #[test]
        fn out_of_bound() {
            let i = UIntIndex::<u8>::default();
            assert_eq!(0, i.get(&2).len());
        }

        #[test]
        fn with_capacity() {
            let mut i = UIntIndex::<u8>::with_capacity(5);
            i.insert(1, 4);
            assert_eq!(2, i.data.len());
            assert_eq!(5, i.data.capacity());
        }

        #[test]
        fn find_eq_many_unique() {
            let l = [0, 1, 2, 3, 4, 5, 6];
            let i = UIntIndex::<u8>::from_list(l);

            assert_eq!(0, i.get_many([]).items_vec(&l).len());
            assert_eq!(0, i.get_many([9]).items_vec(&l).len());
            assert_eq!(vec![&2], i.get_many([2]).items_vec(&l));
            assert_eq!(vec![&6, &2], i.get_many([6, 2]).items_vec(&l));
            assert_eq!(vec![&6, &2], i.get_many([9, 6, 2]).items_vec(&l));
            assert_eq!(vec![&5, &6, &2], i.get_many([5, 9, 6, 2]).items_vec(&l));

            assert_eq!(vec![&2, &3, &4, &5, &6], i.get_many(2..=6).items_vec(&l));
            assert_eq!(vec![&2, &3, &4, &5, &6], i.get_many(2..9).items_vec(&l));
        }

        #[test]
        fn contains() {
            let mut i = UIntIndex::<u8>::default();
            i.insert(5, 5);
            i.insert(2, 2);

            assert!(i.contains(&5));
            assert!(!i.contains(&55));
        }

        #[test]
        fn min() {
            let mut idx = UIntIndex::<u16>::with_capacity(100);
            assert_eq!(0, idx.meta().min_key());
            assert_eq!(0, idx._find_min());

            idx.insert(4, 4);
            assert_eq!(4, idx.meta().min_key());
            assert_eq!(4, idx._find_min());

            idx.insert(2, 8);
            assert_eq!(2, idx.meta().min_key());
            assert_eq!(2, idx._find_min());

            idx.insert(99, 6);
            assert_eq!(2, idx.meta().min_key());
            assert_eq!(2, idx._find_min());
        }

        #[test]
        fn min_rm() {
            let mut idx = UIntIndex::<u16>::with_capacity(100);
            idx.insert(4, 4);
            assert_eq!(4, idx.meta().min_key());
            assert_eq!(4, idx._find_min());

            idx.insert(2, 8);
            assert_eq!(2, idx.meta().min_key());
            assert_eq!(2, idx._find_min());

            // remove min value on Index 2
            *idx.data.get_mut(2).unwrap() = None;
            assert_eq!(2, idx.meta().min_key()); // this cached value is now false
            assert_eq!(4, idx._find_min()); // this is the correct value
        }

        #[test]
        fn max() {
            let mut idx = UIntIndex::<u16>::with_capacity(100);
            assert_eq!(0, idx.meta().max_key());

            idx.insert(4, 4);
            assert_eq!(4, idx.meta().max_key());

            idx.insert(2, 8);
            assert_eq!(4, idx.meta().max_key());

            idx.insert(99, 6);
            assert_eq!(99, idx.meta().max_key());
        }

        #[test]
        fn update() {
            let mut idx = UIntIndex::new();
            idx.insert(2, 4);

            assert_eq!(2, idx.meta().min_key());
            assert_eq!(2, idx.meta().max_key());

            // (old) Key: 99 do not exist, insert a (new) Key 100?
            idx.update(99, 4, 100);
            assert_eq!(101, idx.data.len());
            assert_eq!([4], idx.get(&100));

            // (old) Key 2 exist, but not with Index: 8, insert known Key: 2 with add new Index 8
            idx.update(2, 8, 2);
            assert_eq!([4, 8], idx.get(&2));

            // old Key 2 with Index 8 was removed and (new) Key 4 was added with Index 8
            idx.update(2, 8, 4);
            assert_eq!([8], idx.get(&4));
            assert_eq!([4], idx.get(&2));

            assert_eq!(2, idx.meta().min_key());
            assert_eq!(100, idx.meta().max_key());
        }

        #[test]
        fn delete() {
            let mut idx = UIntIndex::new();
            idx.insert(2, 4);
            idx.insert(2, 3);
            idx.insert(3, 1);

            assert_eq!(2, idx.meta().min_key());
            assert_eq!(3, idx.meta().max_key());

            // delete correct Key with wrong Index, nothing happens
            idx.delete(2, &100);
            assert_eq!([3, 4], idx.get(&2));

            // delete correct Key with correct Index
            idx.delete(2, &3);
            assert_eq!([4], idx.get(&2));
            assert_eq!(2, idx.meta().min_key());
            assert_eq!(3, idx.meta().max_key());

            // delete correct Key with last correct Index, Key now longer exist
            idx.delete(2, &4);
            assert!(idx.get(&2).is_empty());
            assert_eq!(3, idx.meta().min_key());
            assert_eq!(3, idx.meta().max_key());

            idx.insert(2, 4);
            // remove max key
            idx.delete(3, &1);
            assert_eq!(2, idx.meta().max_key());
        }
    }

    mod multi {
        use super::*;

        #[test]
        fn empty() {
            let i = UIntIndex::<u8>::default();
            assert_eq!(0, i.get(&2).len());
            assert!(i.data.is_empty());
        }

        #[test]
        fn find_idx_2() {
            let mut i = UIntIndex::<u8>::default();
            i.insert(2, 2);

            assert_eq!(i.get(&2), [2]);
            assert_eq!(3, i.data.len());
        }

        #[test]
        fn double_index() {
            let mut i = UIntIndex::<u8>::default();
            i.insert(2, 2);
            i.insert(2, 1);

            assert_eq!(i.get(&2), [1, 2]);
        }

        #[test]
        fn find_eq_many_unique() {
            let l = [0, 2, 2, 3, 4, 5, 6];
            let i = UIntIndex::<u8>::from_list(l);

            assert_eq!(0, i.get_many([]).items_vec(&l).len());
            assert_eq!(0, i.get_many([9]).items_vec(&l).len());

            assert_eq!(vec![&2, &2], i.get_many([2]).items_vec(&l));
            assert_eq!(vec![&6, &2, &2], i.get_many([6, 2]).items_vec(&l));
            assert_eq!(vec![&6, &2, &2], i.get_many([9, 6, 2]).items_vec(&l));
            assert_eq!(vec![&5, &6, &2, &2], i.get_many([5, 9, 6, 2]).items_vec(&l));
        }

        #[test]
        fn contains() {
            let mut i = UIntIndex::<u8>::default();
            i.insert(2, 2);
            i.insert(2, 1);

            assert!(i.contains(&2));
            assert!(!i.contains(&55));
        }
    }

    mod keys {
        use super::*;

        #[test]
        fn empty() {
            let keys = UIntIndex::from_iter(Vec::<usize>::new());
            assert!(!keys.exist(&1));
        }

        #[test]
        fn one() {
            let keys = UIntIndex::from_iter([2usize]);
            assert!(!keys.exist(&1));
            assert!(keys.exist(&2));
        }

        #[test]
        fn keys() {
            let keys = UIntIndex::from_iter([5usize, 1, 3]);
            assert_eq!(keys.iter().collect::<Vec<_>>(), vec![&1, &3, &5]);

            let keys = UIntIndex::from_iter([5u8, 1, 3]);
            assert_eq!(keys.iter().collect::<Vec<_>>(), vec![&1, &3, &5]);

            // true is twice, so it will be ignored ones
            let keys = UIntIndex::from_iter([true, false, true]);
            assert_eq!(keys.iter().collect::<Vec<_>>(), vec![&false, &true]);
        }
    }
}
