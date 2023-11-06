use std::{fmt::Debug, marker::PhantomData};

use crate::index::{
    base::IVec,
    indices::{KeyIndex, MultiKeyIndex, UniqueKeyIndex},
    store::{Filterable, MetaData, Store},
};

type UniqueIntIndex<K = usize, X = usize> = IntIndex<UniqueKeyIndex<X>, K, X>;
type MultiIntIndex<K = usize, X = usize> = IntIndex<MultiKeyIndex<X>, K, X>;

#[derive(Debug)]
struct IntIndex<I, K = usize, X = usize> {
    vec: IVec<X, I, (Option<I>, Option<I>)>,
    _key: PhantomData<K>,
    _index: PhantomData<X>,
}

impl<I, K, X> Filterable for IntIndex<I, K, X>
where
    I: KeyIndex<X> + Clone,
    K: Into<i32> + Copy,
{
    type Key = K;
    type Index = X;

    fn contains(&self, key: &Self::Key) -> bool {
        self.vec.contains((*key).into())
    }

    fn get(&self, key: &Self::Key) -> &[Self::Index] {
        self.vec.get((*key).into())
    }
}

impl<I, K, X> Store for IntIndex<I, K, X>
where
    I: KeyIndex<X> + Clone,
    K: Into<i32> + Copy,
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
            _index: PhantomData,
        }
    }
}
impl<I, K, X> Default for IntIndex<I, K, X>
where
    I: KeyIndex<X> + Clone,
{
    fn default() -> Self {
        Self {
            vec: IVec::new(),
            _key: PhantomData,
            _index: PhantomData,
        }
    }
}

impl<I, K, X> MetaData for IntIndex<I, K, X> {
    type Meta<'m> = IntMeta<'m,I,X> where I: 'm, K:'m,X:'m;

    fn meta(&self) -> Self::Meta<'_> {
        IntMeta(&self.vec)
    }
}

pub struct IntMeta<'a, I: 'a, X: 'a>(&'a IVec<X, I, (Option<I>, Option<I>)>);

impl<'s, I, X> IntMeta<'s, I, X>
where
    I: KeyIndex<X> + Clone,
{
    /// Get the smallest (`min`) `Key-Index` which is stored in ``UIntIndex`.
    pub fn min_key_index(&self) -> (Option<usize>, Option<usize>) {
        if let Some(o) = self.0.min_key_index() {
            return o;
        }
        (None, None)
    }

    /// Get the smallest (`max`) `Key-Index` which is stored in ``UIntIndex`.
    pub fn max_key_index(&self) -> (Option<usize>, Option<usize>) {
        if let Some(o) = self.0.max_key_index() {
            return o;
        }
        (None, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::filter::Filter;

    #[test]
    fn insert_plus() {
        let mut i = MultiIntIndex::default();
        i.insert(1, 3);
        i.insert(2, 4);

        assert!(i.contains(&1));
        assert!(!i.contains(&3));

        let r = i.get(&2).iter().collect::<Vec<_>>();
        assert_eq!(vec![&4], r);
    }

    #[test]
    fn insert_minus() {
        let mut i = MultiIntIndex::default();
        i.insert(-1, 3);
        i.insert(-2, 4);

        assert!(i.contains(&-1));
        assert!(!i.contains(&-3));

        let r = i.get(&-2).iter().collect::<Vec<_>>();
        assert_eq!(vec![&4], r);
    }

    #[test]
    fn insert_plus_minus() {
        let mut i = MultiIntIndex::default();
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
        let mut i = MultiIntIndex::default();
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
        let mut i = MultiIntIndex::with_capacity(4);
        i.insert(2, 4);
        i.insert(-2, 3);

        assert_eq!(i.get(&2), [4]);
        assert_eq!(i.get(&-2), [3]);

        i.insert(1, 3);
        let f = Filter(&i);
        assert_eq!([3, 4], (f.eq(&2) | f.eq(&1)));
    }

    #[test]
    fn meta() {
        let mut i = MultiIntIndex::<i8, _>::with_capacity(3);
        i.insert(2, 4);

        assert_eq!((None, Some(2)), i.meta().min_key_index());
        assert_eq!((None, Some(2)), i.meta().max_key_index());

        i.insert(1, 3);
        assert_eq!((None, Some(1)), i.meta().min_key_index());
        assert_eq!((None, Some(2)), i.meta().max_key_index());
    }

    #[test]
    fn index_str() {
        let mut i = MultiIntIndex::<i8, String>::with_capacity(8);
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
            let i = UniqueIntIndex::<i32, usize>::default();
            assert_eq!(0, i.get(&2).len());
            assert!(i.vec.is_empty());
        }

        #[test]
        fn find_idx_2_usize_pos() {
            let mut i = UniqueIntIndex::default();
            i.insert(2, 4);

            assert_eq!(i.get(&2), [4]);
            assert_eq!(4, i.vec.len());
        }

        #[test]
        fn find_idx_2_usize_neg() {
            let mut i = UniqueIntIndex::default();
            i.insert(-2, 4);

            assert_eq!(i.get(&-2), [4]);
            assert_eq!(4, i.vec.len());
        }

        #[test]
        fn find_idx_2_bool() {
            let mut i = UniqueIntIndex::<bool, _>::with_capacity(2);
            i.insert(true, 4);

            assert_eq!(i.get(&true), [4]);
            assert_eq!(2, i.vec.len());
        }

        #[test]
        fn find_idx_2_u16() {
            let mut i = UniqueIntIndex::<u16, _>::with_capacity(2);
            i.insert(2, 4);

            assert_eq!(i.get(&2), [4]);
            assert_eq!(4, i.vec.len());
        }

        #[test]
        fn or_find_idx_3_4() {
            let mut idx = UniqueIntIndex::default();
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
            let mut idx = UniqueIntIndex::default();
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
            let i = UniqueIntIndex::<i32, i32>::default();
            assert_eq!(0, i.get(&2).len());
        }

        #[test]
        fn with_capacity() {
            let mut i = UniqueIntIndex::<u8, _>::with_capacity(5);
            i.insert(1, 4);
            // assert_eq!(2, i.pos_data.len());
            // assert_eq!(5, i.pos_data.capacity());
            // assert_eq!(0, i.neg_data.len());
            // assert_eq!(5, i.neg_data.capacity());
        }

        #[test]
        fn find_eq_many_unique() {
            let l = [0, 1, 2, 3, 4, 5, 6];
            let i = UniqueIntIndex::<u8, _>::from_list(l);

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
            let mut i = UniqueIntIndex::<u8, _>::with_capacity(2);
            i.insert(5, 5);
            i.insert(2, 2);

            assert!(i.contains(&5));
            assert!(!i.contains(&55));
        }

        #[test]
        fn min() {
            let mut idx = UniqueIntIndex::<i16, _>::with_capacity(100);
            assert_eq!((None, None), idx.meta().min_key_index());

            idx.insert(4, 4);
            assert_eq!((None, Some(4)), idx.meta().min_key_index());

            idx.insert(-2, 8);
            assert_eq!((Some(2), None), idx.meta().min_key_index());

            idx.insert(99, 6);
            assert_eq!((Some(2), None), idx.meta().min_key_index());
        }

        // #[test]
        // fn min_rm() {
        //     let mut idx = IntIndex::<u16>::with_capacity(100);
        //     idx.insert(4, 4);
        //     assert_eq!(4, idx.meta().min_key());
        //     assert_eq!(4, idx._find_min());

        //     idx.insert(2, 8);
        //     assert_eq!(2, idx.meta().min_key());
        //     assert_eq!(2, idx._find_min());

        //     idx.delete(2, &8);
        //     assert_eq!(4, idx.meta().min_key()); // this cached value is now false
        //     assert_eq!(4, idx._find_min()); // this is the correct value
        // }

        // #[test]
        // fn max() {
        //     let mut idx = IntIndex::<i16>::with_capacity(100);
        //     assert_eq!(0, idx.meta().max_key());

        //     idx.insert(4, 4);
        //     assert_eq!(4, idx.meta().max_key());

        //     idx.insert(-2, 8);
        //     assert_eq!(4, idx.meta().max_key());

        //     idx.insert(99, 6);
        //     assert_eq!(99, idx.meta().max_key());
        // }

        // #[test]
        // fn update() {
        //     let mut idx = UniqueIntIndex::default();
        //     idx.insert(2, 4);

        //     // assert_eq!(2, idx.meta().min_key());
        //     // assert_eq!(2, idx.meta().max_key());

        //     // (old) Key: 99 do not exist, insert a (new) Key 100?
        //     idx.update(99, 4, 100);
        //     assert_eq!(101, idx.vec.len());
        //     assert_eq!([4], idx.get(&100));

        //     // (old) Key 2 exist, but not with Index: 8, insert known Key: 2 with add new Index 8
        //     idx.update(2, 8, 2);
        //     assert_eq!([4, 8], idx.get(&2));

        //     // old Key 2 with Index 8 was removed and (new) Key 4 was added with Index 8
        //     idx.update(2, 8, 4);
        //     assert_eq!([8], idx.get(&4));
        //     assert_eq!([4], idx.get(&2));

        //     //     assert_eq!(2, idx.meta().min_key());
        //     //     assert_eq!(100, idx.meta().max_key());
        // }

        // #[test]
        // fn delete_empty() {
        //     let idx = IntIndex::new();

        //     assert_eq!(0, idx.meta().min_key());
        //     assert_eq!(0, idx.meta().max_key());
        // }

        // #[test]
        // fn delete_pos() {
        //     let mut idx = UniqueIntIndex::default();
        //     idx.insert(2, 4);
        //     idx.insert(2, 3);
        //     idx.insert(3, 1);

        //     // assert_eq!(2, idx.meta().min_key());
        //     // assert_eq!(3, idx.meta().max_key());

        //     // delete correct Key with wrong Index, nothing happens
        //     idx.delete(2, &100);
        //     assert_eq!([3, 4], idx.get(&2));

        //     // delete correct Key with correct Index
        //     idx.delete(2, &3);
        //     assert_eq!([4], idx.get(&2));
        //     // assert_eq!(2, idx.meta().min_key());
        //     // assert_eq!(3, idx.meta().max_key());

        //     // delete correct Key with last correct Index, Key now longer exist
        //     idx.delete(2, &4);
        //     assert!(idx.get(&2).is_empty());
        //     // assert_eq!(3, idx.meta().min_key());
        //     // assert_eq!(3, idx.meta().max_key());

        //     idx.insert(2, 4);
        //     // remove max key
        //     idx.delete(3, &1);
        //     // assert_eq!(2, idx.meta().max_key());
        // }

        // #[test]
        // fn delete_neg() {
        //     let mut idx = UniqueIntIndex::default();
        //     idx.insert(-2, 4);
        //     idx.insert(-2, 3);
        //     idx.insert(-3, 1);

        //     assert_eq!(-3, idx.meta().min_key());
        //     assert_eq!(-2, idx.meta().max_key());

        //     idx.delete(-3, &1);
        //     assert_eq!(-2, idx.meta().min_key());
        //     assert_eq!(-2, idx.meta().max_key());

        //     idx.insert(-3, 1);
        //     assert_eq!(-3, idx.meta().min_key());
        //     assert_eq!(-2, idx.meta().max_key());

        //     idx.delete(-2, &4);
        //     idx.delete(-2, &3);
        //     assert_eq!(-3, idx.meta().min_key());
        //     assert_eq!(-3, idx.meta().max_key());
        // }

        // #[test]
        // fn delete_pos_neg() {
        //     let mut idx = UniqueIntIndex::default();
        //     idx.insert(2, 4);
        //     idx.insert(-2, 3);
        //     idx.insert(-3, 1);

        //     assert_eq!(-3, idx.meta().min_key());
        //     assert_eq!(2, idx.meta().max_key());

        //     idx.delete(-3, &1);
        //     assert_eq!(-2, idx.meta().min_key());
        //     assert_eq!(2, idx.meta().max_key());

        //     idx.insert(-3, 1);
        //     assert_eq!(-3, idx.meta().min_key());
        //     assert_eq!(2, idx.meta().max_key());

        //     idx.delete(2, &4);
        //     assert_eq!(-3, idx.meta().min_key());
        //     assert_eq!(-2, idx.meta().max_key());
        // }
    }

    // mod multi {
    //     use super::*;

    //     #[test]
    //     fn empty() {
    //         let i = IntIndex::<u8>::with_capacity(2);
    //         assert_eq!(0, i.get(&2).len());
    //         assert!(i.pos_data.is_empty());
    //         assert!(i.neg_data.is_empty());
    //     }

    //     #[test]
    //     fn find_idx_2() {
    //         let mut i = IntIndex::<u8, i32>::with_capacity(2);
    //         i.insert(2, 2);
    //         i.insert(2, -2);

    //         assert_eq!(i.get(&2), [-2, 2]);
    //         assert_eq!(3, i.pos_data.len());
    //         assert_eq!(0, i.neg_data.len());
    //     }

    //     #[test]
    //     fn double_index() {
    //         let mut i = IntIndex::new();
    //         i.insert(2, 2);
    //         i.insert(2, 1);
    //         assert_eq!(i.get(&2), [1, 2]);

    //         i.insert(-2, 2);
    //         i.insert(-2, 1);
    //         assert_eq!(i.get(&-2), [1, 2]);
    //     }

    //     #[test]
    //     fn find_eq_many_unique() {
    //         let l = [0, 2, 2, -3, 4, 5, -6];
    //         let i = IntIndex::<i8>::from_list(l);

    //         assert_eq!(0, i.get_many([]).items_vec(&l).len());
    //         assert_eq!(0, i.get_many([9]).items_vec(&l).len());

    //         assert_eq!(vec![&2, &2], i.get_many([2]).items_vec(&l));
    //         assert_eq!(vec![&-6, &2, &2], i.get_many([-6, 2]).items_vec(&l));
    //         assert_eq!(vec![&-6, &2, &2], i.get_many([9, -6, 2]).items_vec(&l));
    //         assert_eq!(
    //             vec![&5, &-6, &2, &2],
    //             i.get_many([5, 9, -6, 2]).items_vec(&l)
    //         );
    //     }

    //     #[test]
    //     fn contains() {
    //         let mut i = IntIndex::<i8, i8>::with_capacity(3);
    //         i.insert(2, 2);
    //         i.insert(2, 1);
    //         assert!(i.contains(&2));
    //         assert!(!i.contains(&55));

    //         i.insert(-2, -2);
    //         i.insert(-2, -1);
    //         assert!(i.contains(&-2));
    //         assert!(!i.contains(&-55));
    //     }
    // }

    // mod keys {
    //     use super::*;

    //     #[test]
    //     fn empty() {
    //         let keys = IntIndex::from_iter(Vec::<i32>::new());
    //         assert!(!keys.exist(&1));
    //     }

    //     #[test]
    //     fn one() {
    //         let keys = IntIndex::from_iter([2i32]);
    //         assert!(!keys.exist(&1));
    //         assert!(keys.exist(&2));

    //         let keys = IntIndex::from_iter([-2i32]);
    //         assert!(!keys.exist(&-1));
    //         assert!(keys.exist(&-2));
    //     }

    //     #[test]
    //     fn two() {
    //         let keys = IntIndex::from_iter([2i32, -2]);
    //         assert!(!keys.exist(&1));
    //         assert!(keys.exist(&2));
    //         assert!(!keys.exist(&-1));
    //         assert!(keys.exist(&-2));
    //     }

    //     #[test]
    //     fn keys() {
    //         let keys = IntIndex::from_iter([5, 1, 3]);
    //         assert_eq!(keys.iter().collect::<Vec<_>>(), vec![&1, &3, &5]);

    //         let keys = IntIndex::from_iter([5u8, 1, 3]);
    //         assert_eq!(keys.iter().collect::<Vec<_>>(), vec![&1, &3, &5]);

    //         // true is twice, so it will be ignored ones
    //         let keys = IntIndex::from_iter([true, false, true]);
    //         assert_eq!(keys.iter().collect::<Vec<_>>(), vec![&false, &true]);
    //     }

    //     #[test]
    //     fn keys_with_neg() {
    //         let keys = IntIndex::from_iter([5, -1, -3]);
    //         assert_eq!(keys.iter().collect::<Vec<_>>(), vec![&-1, &-3, &5]);

    //         let keys = IntIndex::from_iter([-5, -1, -3]);
    //         assert_eq!(keys.iter().collect::<Vec<_>>(), vec![&-1, &-3, &-5]);

    //         let keys = IntIndex::from_iter([-5, 1, 3, 5]);
    //         assert_eq!(keys.iter().collect::<Vec<_>>(), vec![&-5, &1, &3, &5]);

    //         let keys = IntIndex::from_iter([1, 3, 5, -1, -3, -5]);
    //         assert_eq!(
    //             keys.iter().collect::<Vec<_>>(),
    //             vec![&-1, &-3, &-5, &1, &3, &5]
    //         );
    //     }
    // }
}
