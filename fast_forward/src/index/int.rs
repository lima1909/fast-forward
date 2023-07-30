//! This `Index` is well suitable for `IDs` with [`i32`] compatible data types (for example `Primary Keys`).
//!
use std::{fmt::Debug, marker::PhantomData};

use super::{
    indices::KeyIndices,
    store::{Filterable, Store},
    view::Keys,
};

/// `Key` is from type Into: [`i32`].
#[derive(Debug)]
pub struct IntIndex<K = i32, X = usize> {
    pos_data: Vec<Option<(K, KeyIndices<X>)>>,
    neg_data: Vec<Option<(K, KeyIndices<X>)>>,
    _key: PhantomData<K>,
}

impl<K, X> Filterable for IntIndex<K, X>
where
    K: Into<i32> + TryInto<usize> + Copy,
{
    type Key = K;
    type Index = X;

    #[inline]
    fn get(&self, key: &Self::Key) -> &[X] {
        let ikey: i32 = (*key).into();
        match self.data(ikey).get(pos(ikey)) {
            Some(Some((_, idx))) => idx.as_slice(),
            _ => &[],
        }
    }

    #[inline]
    fn contains(&self, key: &Self::Key) -> bool {
        let ikey: i32 = (*key).into();
        matches!(self.data(ikey).get(pos(ikey)), Some(Some(_)))
    }
}

impl<K, X> Store for IntIndex<K, X>
where
    K: Into<i32> + TryInto<usize> + Copy,
    X: Ord + Clone,
{
    fn insert(&mut self, key: K, x: X) {
        let ikey: i32 = key.into();
        let pos = pos(ikey);
        let data = self.data_mut(ikey);

        if data.len() <= pos {
            data.resize(pos + 1, None);
        }

        match data[pos].as_mut() {
            Some((_, idx)) => idx.add(x),
            None => data[pos] = Some((key, KeyIndices::new(x))),
        }
    }

    fn delete(&mut self, key: K, x: &X) {
        let ikey: i32 = key.into();
        let data = self.data_mut(ikey);

        if let Some(Some((_, rm_idx))) = data.get_mut(pos(ikey)) {
            // if the Index is the last, then remove complete Index
            if rm_idx.remove(x).is_empty() {
                data[pos(ikey)] = None
            }
        }
    }

    fn with_capacity(capacity: usize) -> Self {
        IntIndex {
            pos_data: Vec::with_capacity(capacity),
            neg_data: Vec::with_capacity(capacity),
            _key: PhantomData,
        }
    }
}

impl<K, X> IntIndex<K, X> {
    #[inline]
    fn data(&self, key: i32) -> &[Option<(K, KeyIndices<X>)>] {
        if key < 0 {
            return &self.neg_data;
        }
        &self.pos_data
    }

    #[inline]
    fn data_mut(&mut self, key: i32) -> &mut Vec<Option<(K, KeyIndices<X>)>> {
        if key < 0 {
            return &mut self.neg_data;
        }
        &mut self.pos_data
    }
}

impl<K> Keys for IntIndex<K>
where
    K: Into<i32> + TryInto<usize> + Copy,
{
    type Key = K;

    fn exist(&self, key: &K) -> bool {
        let key: i32 = (*key).into();
        matches!(self.data(key).get(pos(key)), Some(Some(_)))
    }

    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = &'a Self::Key> + 'a> {
        Box::new(KeyIntIter::new(&self.neg_data, &self.pos_data))
    }

    fn from_iter<I>(it: I) -> Self
    where
        I: IntoIterator<Item = K>,
    {
        fn add_key<K>(view: &mut IntIndex<K>, key: K)
        where
            K: Into<i32> + TryInto<usize> + Copy,
        {
            let ikey: i32 = key.into();
            let pos = pos(ikey);
            let data = view.data_mut(ikey);

            if data.len() <= pos {
                data.resize(pos + 1, None);
            }

            data[pos] = Some((key, KeyIndices::empty()))
        }

        let v = Vec::from_iter(it);
        let mut view =
            Self::with_capacity(v.iter().map(|k| pos((*k).into())).max().unwrap_or_default());
        v.into_iter().for_each(|key| add_key(&mut view, key));
        view
    }
}

#[inline]
fn pos(key: i32) -> usize {
    if key < 0 {
        key.abs().try_into()
    } else {
        key.try_into()
    }
    .expect("expect key, which can convert into: i32")
}

struct KeyIntIter<'a, K> {
    pos: &'a [Option<(K, KeyIndices)>],
    iter: std::slice::Iter<'a, Option<(K, KeyIndices)>>,
    is_neg: bool,
}

impl<'a, K> KeyIntIter<'a, K> {
    fn new(neg: &'a [Option<(K, KeyIndices)>], pos: &'a [Option<(K, KeyIndices)>]) -> Self {
        Self {
            pos,
            is_neg: true,
            iter: neg.iter(),
        }
    }
}

impl<'a, K> Iterator for KeyIntIter<'a, K> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(opt) = self.iter.by_ref().flatten().next() {
            return Some(&opt.0);
        }

        if self.is_neg {
            self.is_neg = false;
            self.iter = self.pos.iter();

            if let Some(opt) = self.iter.by_ref().flatten().next() {
                return Some(&opt.0);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::filter::Filter;

    impl IntIndex<i32> {
        fn new() -> Self {
            Self {
                pos_data: Vec::new(),
                neg_data: Vec::new(),
                _key: PhantomData,
            }
        }
    }

    #[test]
    fn insert_plus() {
        let mut i = IntIndex::new();
        i.insert(1, 3);
        i.insert(2, 4);

        assert!(i.contains(&1));
        assert!(!i.contains(&3));

        let r = i.get(&2).iter().collect::<Vec<_>>();
        assert_eq!(vec![&4], r);
    }

    #[test]
    fn insert_minus() {
        let mut i = IntIndex::new();
        i.insert(-1, 3);
        i.insert(-2, 4);

        assert!(i.contains(&-1));
        assert!(!i.contains(&-3));

        let r = i.get(&-2).iter().collect::<Vec<_>>();
        assert_eq!(vec![&4], r);
    }

    #[test]
    fn insert_plus_minus() {
        let mut i = IntIndex::new();
        i.insert(1, 3);
        i.insert(-2, 4);
        i.insert(3, 8);

        assert!(i.contains(&1));
        assert!(i.contains(&-2));
        assert!(i.contains(&3));
        assert!(!i.contains(&5));

        let r = i.get_many([-2, 3]).collect::<Vec<_>>();
        assert_eq!(vec![&4, &8], r);
    }

    #[test]
    fn delete_plus_minus() {
        let mut i = IntIndex::new();
        i.insert(1, 3);
        i.insert(-2, 4);
        i.insert(1, 5);

        assert!(i.contains(&1));
        assert!(i.contains(&-2));

        i.delete(1, &3);
        assert!(i.contains(&1));
        assert!(i.contains(&-2));

        i.delete(1, &5);
        assert!(!i.contains(&1));
        assert!(i.contains(&-2));
    }

    #[test]
    fn filter() {
        let mut i = IntIndex::with_capacity(4);
        i.insert(2, 4);

        assert_eq!(i.get(&2), [4]);

        i.insert(1, 3);
        let f = Filter(&i);
        assert_eq!([3, 4], (f.eq(&2) | f.eq(&1)));
    }

    // #[test]
    // fn meta() {
    //     let mut i = UIntIndex::new();
    //     i.insert(2, 4);

    //     assert_eq!(2, i.meta().min());
    //     assert_eq!(2, i.meta().max());

    //     i.insert(1, 3);
    //     assert_eq!(1, i.meta().min());
    //     assert_eq!(2, i.meta().max());
    // }

    #[test]
    fn index_str() {
        let mut i = IntIndex::<i8, String>::with_capacity(8);
        i.insert(1, "Jasmin".into());
        i.insert(2, "Mario 1".into());
        i.insert(2, "Mario 2".into());
        i.insert(-5, "Paul".into());

        assert!(i.contains(&-5));

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

        let r = i.get_many([1, -5]).collect::<Vec<_>>();
        assert_eq!(vec![&String::from("Jasmin"), &String::from("Paul")], r);

        let r = i.get_many([-5, 1]).collect::<Vec<_>>();
        assert_eq!(vec![&String::from("Paul"), &String::from("Jasmin")], r);
    }

    mod unique {
        use super::*;

        #[test]
        fn empty() {
            let i = IntIndex::new();
            assert_eq!(0, i.get(&2).len());
            assert!(i.pos_data.is_empty());
            assert!(i.neg_data.is_empty());
        }

        #[test]
        fn find_idx_2_usize_pos() {
            let mut i = IntIndex::new();
            i.insert(2, 4);

            assert_eq!(i.get(&2), [4]);
            assert_eq!(3, i.pos_data.len());
            assert_eq!(0, i.neg_data.len());
        }

        #[test]
        fn find_idx_2_usize_neg() {
            let mut i = IntIndex::new();
            i.insert(-2, 4);

            assert_eq!(i.get(&-2), [4]);
            assert_eq!(0, i.pos_data.len());
            assert_eq!(3, i.neg_data.len());
        }

        #[test]
        fn find_idx_2_bool() {
            let mut i = IntIndex::<bool>::with_capacity(2);
            i.insert(true, 4);

            assert_eq!(i.get(&true), [4]);
            assert_eq!(2, i.pos_data.len());
            assert_eq!(0, i.neg_data.len());
        }

        #[test]
        fn find_idx_2_u16() {
            let mut i = IntIndex::<u16>::with_capacity(2);
            i.insert(2, 4);

            assert_eq!(i.get(&2), [4]);
            assert_eq!(3, i.pos_data.len());
            assert_eq!(0, i.neg_data.len());
        }

        #[test]
        fn or_find_idx_3_4() {
            let mut idx = IntIndex::new();
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
            let mut idx = IntIndex::new();
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
            let i = IntIndex::new();
            assert_eq!(0, i.get(&2).len());
        }

        #[test]
        fn with_capacity() {
            let mut i = IntIndex::<u8>::with_capacity(5);
            i.insert(1, 4);
            assert_eq!(2, i.pos_data.len());
            assert_eq!(5, i.pos_data.capacity());
            assert_eq!(0, i.neg_data.len());
            assert_eq!(5, i.neg_data.capacity());
        }

        #[test]
        fn find_eq_many_unique() {
            let l = [0, 1, 2, 3, 4, 5, 6];
            let i = IntIndex::<u8>::from_list(l);

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
            let mut i = IntIndex::<u8>::with_capacity(2);
            i.insert(5, 5);
            i.insert(2, 2);

            assert!(i.contains(&5));
            assert!(!i.contains(&55));
        }

        //     #[test]
        //     fn min() {
        //         let mut idx = UIntIndex::<u16>::with_capacity(100);
        //         assert_eq!(0, idx.min());
        //         assert_eq!(0, idx._find_min());

        //         idx.insert(4, 4);
        //         assert_eq!(4, idx.min());
        //         assert_eq!(4, idx._find_min());

        //         idx.insert(2, 8);
        //         assert_eq!(2, idx.min());
        //         assert_eq!(2, idx._find_min());

        //         idx.insert(99, 6);
        //         assert_eq!(2, idx.min());
        //         assert_eq!(2, idx._find_min());
        //     }

        //     #[test]
        //     fn min_rm() {
        //         let mut idx = UIntIndex::<u16>::with_capacity(100);
        //         idx.insert(4, 4);
        //         assert_eq!(4, idx.min());
        //         assert_eq!(4, idx._find_min());

        //         idx.insert(2, 8);
        //         assert_eq!(2, idx.min());
        //         assert_eq!(2, idx._find_min());

        //         // remove min value on Index 2
        //         *idx.data.get_mut(2).unwrap() = None;
        //         assert_eq!(2, idx.min()); // this cached value is now false
        //         assert_eq!(4, idx._find_min()); // this is the correct value
        //     }

        //     #[test]
        //     fn max() {
        //         let mut idx = UIntIndex::<u16>::with_capacity(100);
        //         assert_eq!(0, idx.max());

        //         idx.insert(4, 4);
        //         assert_eq!(4, idx.max());

        //         idx.insert(2, 8);
        //         assert_eq!(4, idx.max());

        //         idx.insert(99, 6);
        //         assert_eq!(99, idx.max());
        //     }

        #[test]
        fn update() {
            let mut idx = IntIndex::new();
            idx.insert(2, 4);

            // assert_eq!(2, idx.min());
            // assert_eq!(2, idx.max());

            // (old) Key: 99 do not exist, insert a (new) Key 100?
            idx.update(99, 4, 100);
            assert_eq!(101, idx.pos_data.len());
            assert_eq!([4], idx.get(&100));

            // (old) Key 2 exist, but not with Index: 8, insert known Key: 2 with add new Index 8
            idx.update(2, 8, 2);
            assert_eq!([4, 8], idx.get(&2));

            // old Key 2 with Index 8 was removed and (new) Key 4 was added with Index 8
            idx.update(2, 8, 4);
            assert_eq!([8], idx.get(&4));
            assert_eq!([4], idx.get(&2));

            // assert_eq!(2, idx.min());
            // assert_eq!(100, idx.max());
        }

        #[test]
        fn delete() {
            let mut idx = IntIndex::new();
            idx.insert(2, 4);
            idx.insert(2, 3);
            idx.insert(3, 1);

            // assert_eq!(2, idx.min());
            // assert_eq!(3, idx.max());

            // delete correct Key with wrong Index, nothing happens
            idx.delete(2, &100);
            assert_eq!([3, 4], idx.get(&2));

            // delete correct Key with correct Index
            idx.delete(2, &3);
            assert_eq!([4], idx.get(&2));

            // delete correct Key with last correct Index, Key now longer exist
            idx.delete(2, &4);
            assert!(idx.get(&2).is_empty());

            // assert_eq!(3, idx.min());
            // assert_eq!(3, idx.max());
        }
    }

    mod multi {
        use super::*;

        #[test]
        fn empty() {
            let i = IntIndex::<u8>::with_capacity(2);
            assert_eq!(0, i.get(&2).len());
            assert!(i.pos_data.is_empty());
            assert!(i.neg_data.is_empty());
        }

        #[test]
        fn find_idx_2() {
            let mut i = IntIndex::<u8, i32>::with_capacity(2);
            i.insert(2, 2);
            i.insert(2, -2);

            assert_eq!(i.get(&2), [-2, 2]);
            assert_eq!(3, i.pos_data.len());
            assert_eq!(0, i.neg_data.len());
        }

        #[test]
        fn double_index() {
            let mut i = IntIndex::new();
            i.insert(2, 2);
            i.insert(2, 1);
            assert_eq!(i.get(&2), [1, 2]);

            i.insert(-2, 2);
            i.insert(-2, 1);
            assert_eq!(i.get(&-2), [1, 2]);
        }

        #[test]
        fn find_eq_many_unique() {
            let l = [0, 2, 2, -3, 4, 5, -6];
            let i = IntIndex::<i8>::from_list(l);

            assert_eq!(0, i.get_many([]).items_vec(&l).len());
            assert_eq!(0, i.get_many([9]).items_vec(&l).len());

            assert_eq!(vec![&2, &2], i.get_many([2]).items_vec(&l));
            assert_eq!(vec![&-6, &2, &2], i.get_many([-6, 2]).items_vec(&l));
            assert_eq!(vec![&-6, &2, &2], i.get_many([9, -6, 2]).items_vec(&l));
            assert_eq!(
                vec![&5, &-6, &2, &2],
                i.get_many([5, 9, -6, 2]).items_vec(&l)
            );
        }

        #[test]
        fn contains() {
            let mut i = IntIndex::<i8, i8>::with_capacity(3);
            i.insert(2, 2);
            i.insert(2, 1);
            assert!(i.contains(&2));
            assert!(!i.contains(&55));

            i.insert(-2, -2);
            i.insert(-2, -1);
            assert!(i.contains(&-2));
            assert!(!i.contains(&-55));
        }
    }

    mod keys {
        use super::*;

        #[test]
        fn empty() {
            let keys = IntIndex::from_iter(Vec::<i32>::new());
            assert!(!keys.exist(&1));
        }

        #[test]
        fn one() {
            let keys = IntIndex::from_iter([2i32]);
            assert!(!keys.exist(&1));
            assert!(keys.exist(&2));

            let keys = IntIndex::from_iter([-2i32]);
            assert!(!keys.exist(&-1));
            assert!(keys.exist(&-2));
        }

        #[test]
        fn two() {
            let keys = IntIndex::from_iter([2i32, -2]);
            assert!(!keys.exist(&1));
            assert!(keys.exist(&2));
            assert!(!keys.exist(&-1));
            assert!(keys.exist(&-2));
        }

        #[test]
        fn keys() {
            let keys = IntIndex::from_iter([5, 1, 3]);
            assert_eq!(keys.iter().collect::<Vec<_>>(), vec![&1, &3, &5]);

            let keys = IntIndex::from_iter([5u8, 1, 3]);
            assert_eq!(keys.iter().collect::<Vec<_>>(), vec![&1, &3, &5]);

            // true is twice, so it will be ignored ones
            let keys = IntIndex::from_iter([true, false, true]);
            assert_eq!(keys.iter().collect::<Vec<_>>(), vec![&false, &true]);
        }

        #[test]
        fn keys_with_neg() {
            let keys = IntIndex::from_iter([5, -1, -3]);
            assert_eq!(keys.iter().collect::<Vec<_>>(), vec![&-1, &-3, &5]);

            let keys = IntIndex::from_iter([-5, -1, -3]);
            assert_eq!(keys.iter().collect::<Vec<_>>(), vec![&-1, &-3, &-5]);

            let keys = IntIndex::from_iter([-5, 1, 3, 5]);
            assert_eq!(keys.iter().collect::<Vec<_>>(), vec![&-5, &1, &3, &5]);

            let keys = IntIndex::from_iter([1, 3, 5, -1, -3, -5]);
            assert_eq!(
                keys.iter().collect::<Vec<_>>(),
                vec![&-1, &-3, &-5, &1, &3, &5]
            );
        }
    }
}