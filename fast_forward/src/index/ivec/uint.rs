//! This `Index` is well suitable for `IDs` with [`usize`] compatible data types (for example `Primary Keys`).
//!
use std::{fmt::Debug, marker::PhantomData};

use crate::index::{
    indices::{KeyIndex, MultiKeyIndex, UniqueKeyIndex},
    ivec::IVec,
    store::{Filterable, MetaData, Store, View, ViewCreator},
};

pub type UniqueUIntIndex<K = usize, X = usize> = UIntIndex<UniqueKeyIndex<X>, K, X>;
pub type MultiUIntIndex<K = usize, X = usize> = UIntIndex<MultiKeyIndex<X>, K, X>;

/// `Key` is from type [`usize`] and the information are saved in a List (Store).
#[derive(Debug)]
#[repr(transparent)]
pub struct UIntIndex<I, K = usize, X = usize> {
    vec: IVec<I, K, X, Option<I>>,
    _key: PhantomData<K>,
}

impl<I, K, X> Filterable for UIntIndex<I, K, X>
where
    I: KeyIndex<X>,
    K: Into<usize> + Copy,
{
    type Key = K;
    type Index = X;

    fn contains(&self, key: &Self::Key) -> bool {
        self.vec.contains_key((*key).into())
    }

    fn get(&self, key: &Self::Key) -> &[Self::Index] {
        self.vec.get_indeces_by_key((*key).into())
    }
}

impl<'a, I, K, X> ViewCreator<'a> for UIntIndex<I, K, X>
where
    I: KeyIndex<X> + 'a,
    K: Into<usize>,
{
    type Key = K;
    type Filter = IVec<I, usize, X, Option<&'a I>>;

    fn create_view<It>(&'a self, keys: It) -> View<Self::Filter>
    where
        It: IntoIterator<Item = Self::Key>,
    {
        let mut view = Self::Filter::new();
        view.vec.resize(self.vec.len(), None);

        for key in keys {
            let idx: usize = key.into();
            if let Some(opt) = self.vec.get(idx) {
                view[idx] = opt.as_ref();
            }
        }

        View(view)
    }
}

impl<I, K, X> Store for UIntIndex<I, K, X>
where
    I: KeyIndex<X> + Clone,
    K: Into<usize> + Copy,
{
    fn insert(&mut self, key: Self::Key, idx: Self::Index) {
        self.vec.insert(key.into(), idx)
    }

    fn delete(&mut self, key: Self::Key, idx: &Self::Index) {
        self.vec.delete(key.into(), idx)
    }

    fn with_capacity(capacity: usize) -> Self {
        Self {
            vec: IVec::with_capacity(capacity),
            _key: PhantomData,
        }
    }
}

impl<I, K, X> Default for UIntIndex<I, K, X>
where
    I: KeyIndex<X>,
{
    fn default() -> Self {
        Self {
            vec: IVec::new(),
            _key: PhantomData,
        }
    }
}

impl<I, K, X> MetaData for UIntIndex<I, K, X> {
    type Meta<'m> = UIntMeta<'m, I,K,X> where I:'m,K:'m,X:'m;

    fn meta(&self) -> Self::Meta<'_> {
        UIntMeta(&self.vec)
    }
}

pub struct UIntMeta<'a, I: 'a, K, X: 'a>(&'a IVec<I, K, X, Option<I>>);

impl<'s, I, K, X> UIntMeta<'s, I, K, X>
where
    I: KeyIndex<X>,
{
    /// Get the smallest (`min`) `Key-Index` which is stored in `UIntIndex`.
    pub fn min_key_index(&self) -> Option<usize> {
        self.0.min_key_index()
    }

    /// Get the smallest (`max`) `Key-Index` which is stored in `UIntIndex`.
    pub fn max_key_index(&self) -> Option<usize> {
        self.0.max_key_index()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        super::super::{
            filter::Filter,
            store::{Store, ViewCreator},
        },
        *,
    };

    impl UIntIndex<MultiKeyIndex<usize>, usize, usize> {
        fn new() -> Self {
            Self::default()
        }
    }

    #[test]
    fn create_view() {
        let mut i = MultiUIntIndex::<u8, u8>::default();
        i.insert(1, 2);
        i.insert(2, 4);
        i.insert(2, 5);
        i.insert(3, 6);
        i.insert(4, 8);
        i.insert(4, 9);
        i.insert(5, 10);

        let view = i.create_view([1, 2, 4]);
        assert!(view.contains(&1));
        assert!(view.contains(&4));
        assert!(!view.contains(&100));

        assert_eq!(view.get(&2), &[4, 5]);
        assert_eq!(view.get(&4), &[8, 9]);
        assert_eq!(view.get(&100), &[]);

        assert_eq!(
            view.get_many([2, 4]).collect::<Vec<_>>(),
            vec![&4, &5, &8, &9]
        );

        assert!(!view.contains(&5));

        i.update(2, 5, 4);
        i.update(4, 99, 4);

        let view = i.create_view([1, 2, 4, 100]);
        assert_eq!(view.get(&2), &[4]);
        assert_eq!(view.get(&4), &[5, 8, 9, 99]);
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
    fn index_str() {
        let mut i = UIntIndex::<MultiKeyIndex<String>, usize, String>::default();
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
            let i = UniqueUIntIndex::<usize, usize>::default();
            assert_eq!(0, i.get(&2).len());
            assert!(i.vec.is_empty());
        }

        #[test]
        #[should_panic]
        fn add_twice() {
            let mut i = UniqueUIntIndex::<usize, usize>::default();
            i.insert(2, 4);
            i.insert(2, 4);
        }

        #[test]
        fn find_idx_2_usize() {
            let mut i = UniqueUIntIndex::<usize, usize>::default();
            i.insert(2, 4);

            assert_eq!(i.get(&2), [4]);
            assert_eq!(4, i.vec.len());
        }

        #[test]
        fn find_idx_2_bool() {
            let mut i = UniqueUIntIndex::<bool, _>::default();
            i.insert(true, 4);

            assert_eq!(i.get(&true), [4]);
            assert_eq!(2, i.vec.len());
        }

        #[test]
        fn find_idx_2_u16() {
            let mut i = UniqueUIntIndex::<u16, _>::default();
            i.insert(2, 4);

            assert_eq!(i.get(&2), [4]);
            assert_eq!(4, i.vec.len());
        }

        #[test]
        fn or_find_idx_3_4() {
            let mut idx = UniqueUIntIndex::<usize, _>::default();
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
            let mut idx = UniqueUIntIndex::<usize, _>::default();
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
            let i = UniqueUIntIndex::<u8, u8>::default();
            assert_eq!(0, i.get(&2).len());
        }

        #[test]
        fn with_capacity() {
            let mut i = UniqueUIntIndex::<u8, _>::with_capacity(5);
            i.insert(1, 4);
            assert_eq!(2, i.vec.len());
            assert_eq!(5, i.vec.capacity());
        }

        #[test]
        fn find_eq_many_unique() {
            let l = [0, 1, 2, 3, 4, 5, 6];
            let i = UniqueUIntIndex::<u8, _>::from_list(l);

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
            let mut i = UniqueUIntIndex::<u8, _>::default();
            i.insert(5, 5);
            i.insert(2, 2);

            assert!(i.contains(&5));
            assert!(!i.contains(&55));
        }

        #[test]
        fn min() {
            let mut idx = UniqueUIntIndex::<u16, _>::with_capacity(100);
            assert_eq!(None, idx.meta().min_key_index());

            idx.insert(4, 4);
            assert_eq!(Some(4), idx.meta().min_key_index());

            idx.insert(2, 8);
            assert_eq!(Some(2), idx.meta().min_key_index());

            idx.insert(99, 6);
            assert_eq!(Some(2), idx.meta().min_key_index());
        }

        #[test]
        fn min_rm() {
            let mut idx = UniqueUIntIndex::<u16, _>::with_capacity(100);
            idx.insert(4, 4);
            assert_eq!(Some(4), idx.meta().min_key_index());

            idx.insert(2, 8);
            assert_eq!(Some(2), idx.meta().min_key_index());

            // remove min value on Index 2
            idx.delete(2, &8);
            assert_eq!(Some(4), idx.meta().min_key_index()); // this cached value is now false
        }

        #[test]
        fn max() {
            let mut idx = UniqueUIntIndex::<u8, _>::with_capacity(100);
            assert_eq!(None, idx.meta().max_key_index());

            idx.insert(4, 4);
            assert_eq!(Some(4), idx.meta().max_key_index());

            idx.insert(2, 8);
            assert_eq!(Some(4), idx.meta().max_key_index());

            idx.insert(99, 6);
            assert_eq!(Some(99), idx.meta().max_key_index());
        }

        #[test]
        fn update() {
            let mut idx = UniqueUIntIndex::<usize, usize>::default();
            idx.insert(2, 4);

            assert_eq!(Some(2), idx.meta().min_key_index());
            assert_eq!(Some(2), idx.meta().max_key_index());

            // (old) Key: 99 do not exist, insert a (new) Key 100?
            idx.update(99, 4, 100);
            assert_eq!(200, idx.vec.len());
            assert_eq!([4], idx.get(&100));

            // (old) Key 2 exist, but not with Index: 8, insert known Key: 2 with add new Index 8
            // idx.update(2, 8, 2);
            // assert_eq!([4, 8], idx.get(&2));

            // old Key 2 with Index 8 was removed and (new) Key 4 was added with Index 8
            idx.update(2, 8, 4);
            assert_eq!([8], idx.get(&4));
            assert_eq!([4], idx.get(&2));

            assert_eq!(Some(2), idx.meta().min_key_index());
            assert_eq!(Some(100), idx.meta().max_key_index());
        }

        #[test]
        fn delete() {
            let mut idx = UniqueUIntIndex::<usize, _>::default();
            idx.insert(2, 4);
            idx.insert(3, 1);

            assert_eq!(2, idx.meta().min_key_index().unwrap());
            assert_eq!(3, idx.meta().max_key_index().unwrap());

            // delete correct Key with wrong Index, nothing happens
            idx.delete(2, &100);
            assert_eq!([4], idx.get(&2));

            // delete correct Key with correct Index
            idx.delete(2, &4);
            assert_eq!(None, idx.get(&2).iter().next());
            assert_eq!(3, idx.meta().min_key_index().unwrap());
            assert_eq!(3, idx.meta().max_key_index().unwrap());

            idx.insert(2, 4);
            // remove max key
            idx.delete(3, &1);
            assert_eq!(2, idx.meta().max_key_index().unwrap());
        }
    }

    mod multi {
        use super::*;

        #[test]
        fn empty() {
            let i = MultiUIntIndex::<u8, u8>::default();
            assert_eq!(0, i.get(&2).len());
            assert!(i.vec.is_empty());
        }

        #[test]
        fn find_idx_2() {
            let mut i = MultiUIntIndex::<u8, _>::default();
            i.insert(2, 2);

            assert_eq!(i.get(&2), [2]);
            assert_eq!(4, i.vec.len());
        }

        #[test]
        fn double_index() {
            let mut i = MultiUIntIndex::<u8, _>::default();
            i.insert(2, 2);
            i.insert(2, 1);

            assert_eq!(i.get(&2), [1, 2]);
        }

        #[test]
        fn find_eq_many_unique() {
            let l = [0, 2, 2, 3, 4, 5, 6];
            let i = MultiUIntIndex::<u8, _>::from_list(l);

            assert_eq!(0, i.get_many([]).items_vec(&l).len());
            assert_eq!(0, i.get_many([9]).items_vec(&l).len());

            assert_eq!(vec![&2, &2], i.get_many([2]).items_vec(&l));
            assert_eq!(vec![&6, &2, &2], i.get_many([6, 2]).items_vec(&l));
            assert_eq!(vec![&6, &2, &2], i.get_many([9, 6, 2]).items_vec(&l));
            assert_eq!(vec![&5, &6, &2, &2], i.get_many([5, 9, 6, 2]).items_vec(&l));
        }

        #[test]
        fn contains() {
            let mut i = MultiUIntIndex::<u8, _>::default();
            i.insert(2, 2);
            i.insert(2, 1);

            assert!(i.contains(&2));
            assert!(!i.contains(&55));
        }

        #[test]
        fn delete() {
            let mut idx = MultiUIntIndex::default();
            idx.insert(2usize, 4);
            idx.insert(2, 3);
            idx.insert(3, 1);

            assert_eq!(Some(2), idx.meta().min_key_index());
            assert_eq!(Some(3), idx.meta().max_key_index());

            // delete correct Key with wrong Index, nothing happens
            idx.delete(2, &100);
            assert_eq!([3, 4], idx.get(&2));

            // delete correct Key with correct Index
            idx.delete(2, &3);
            assert_eq!([4], idx.get(&2));
            assert_eq!(Some(2), idx.meta().min_key_index());
            assert_eq!(Some(3), idx.meta().max_key_index());

            // delete correct Key with last correct Index, Key now longer exist
            idx.delete(2, &4);
            assert!(idx.get(&2).is_empty());
            assert_eq!(Some(3), idx.meta().min_key_index());
            assert_eq!(Some(3), idx.meta().max_key_index());

            idx.insert(2, 4);
            // remove max key
            idx.delete(3, &1);
            assert_eq!(Some(2), idx.meta().max_key_index());
        }
    }

    //     mod keys {
    //         // use super::*;

    //         // #[test]
    //         // fn empty() {
    //         //     let keys = UIntIndex::from_iter(Vec::<usize>::new());
    //         //     assert!(!keys.exist(&1));
    //         // }

    //         // #[test]
    //         // fn one() {
    //         //     let keys = UIntIndex::from_iter([2usize]);
    //         //     assert!(!keys.exist(&1));
    //         //     assert!(keys.exist(&2));
    //         // }

    //         // #[test]
    //         // fn keys() {
    //         //     let keys = UIntIndex::from_iter([5usize, 1, 3]);
    //         //     assert_eq!(keys.iter().collect::<Vec<_>>(), vec![&1, &3, &5]);

    //         //     let keys = UIntIndex::from_iter([5u8, 1, 3]);
    //         //     assert_eq!(keys.iter().collect::<Vec<_>>(), vec![&1, &3, &5]);

    //         //     // true is twice, so it will be ignored ones
    //         //     let keys = UIntIndex::from_iter([true, false, true]);
    //         //     assert_eq!(keys.iter().collect::<Vec<_>>(), vec![&false, &true]);
    //         // }
    // }
}
