//! This `Index` is well suitable for `IDs` with [`usize`] compatible data types (for example `Primary Keys`).
//!
//! Performance: The `eq - filter` is: O(1).
//! The `Key` is the position (index) in the Vec..
//!
//!```text
//! let _list_numbers_unique = vec![3, 2, 4, 1, ...];
//!
//! Unique-Index:
//!
//!  Key | Idx (_values)
//! --------------------
//!  0   |  -
//!  1   |  3
//!  2   |  1
//!  3   |  0
//!  4   |  2
//! ...  | ...
//!
//!
//! let _list_numbers_multi = vec![3, 2, 3, 1, 2, 2, ...];
//!
//! Muli-Index:
//!
//!  Key | Idx (_values)
//! --------------------
//!  0   |  -
//!  1   |  3
//!  2   |  1, 4, 5
//!  3   |  0, 2
//! ...  | ...
//! ```
use crate::{
    index::{Index, Store},
    Idx, EMPTY_IDXS,
};
use std::{borrow::Cow, marker::PhantomData};

use super::{Equals, MinMax};

/// `Key` is from type [`usize`] and the information are saved in a List (Store).
#[derive(Debug, Default)]
pub struct UIntIndex<K: Default = usize> {
    data: Vec<Option<Index>>,
    min_max_cache: MinMax<usize>,
    _key: PhantomData<K>,
}

impl UIntIndex<usize> {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            min_max_cache: MinMax::default(),
            _key: PhantomData,
        }
    }
}

impl<K> Store<K> for UIntIndex<K>
where
    K: Default + Into<usize>,
{
    fn insert(&mut self, k: K, i: Idx) {
        let k = k.into();

        if self.data.len() <= k {
            self.data.resize(k + 1, None);
        }

        match self.data[k].as_mut() {
            Some(idx) => idx.add(i),
            None => self.data[k] = Some(Index::new(i)),
        }

        self.min_max_cache.new_min(k);
        self.min_max_cache.new_max(k);
    }

    fn with_capacity(capacity: usize) -> Self {
        UIntIndex {
            data: Vec::with_capacity(capacity),
            min_max_cache: MinMax::default(),
            _key: PhantomData,
        }
    }
}

impl<K> Equals<K> for UIntIndex<K>
where
    K: Default + Into<usize>,
{
    #[inline]
    fn eq(&self, key: K) -> Cow<[Idx]> {
        match &self.data.get(key.into()) {
            Some(Some(idx)) => idx.get(),
            _ => Cow::Borrowed(EMPTY_IDXS),
        }
    }
}

impl<K> UIntIndex<K>
where
    K: Default,
{
    /// Filter for get the smallest (`min`) `Key` which is stored in `UIntIndex`.
    pub fn min(&self) -> usize {
        self.min_max_cache.min
    }

    /// Filter for get the highest (`max`) `Key` which is stored in `UIntIndex`.
    pub fn max(&self) -> usize {
        self.min_max_cache.max
    }

    /// Find `min` key. _Importand_ if the min value was removed, to find the new valid `min Key`.
    fn _find_min(&self) -> usize {
        self.data
            .iter()
            .enumerate()
            .find(|(_i, item)| item.is_some())
            .map(|(pos, _item)| pos)
            .unwrap_or_default()
    }

    /// Find `max` key. _Importand_ if the max value was removed, to find the new valid `max Key`.
    fn _find_max(&self) -> usize {
        match self.data.last() {
            Some(Some(_)) => self.data.len() - 1,
            _ => 0,
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
            let i = UIntIndex::new();
            assert_eq!(0, i.eq(2).len());
            assert!(i.data.is_empty());
        }

        #[test]
        fn find_idx_2_usize() {
            let mut i = UIntIndex::new();
            i.insert(2, 4);

            assert_eq!(*i.eq(2), [4]);
            assert_eq!(3, i.data.len());
        }

        #[test]
        fn find_idx_2_bool() {
            let mut i = UIntIndex::<bool>::default();
            i.insert(true, 4);

            assert_eq!(*i.eq(true), [4]);
            assert_eq!(2, i.data.len());
        }

        #[test]
        fn find_idx_2_u16() {
            let mut i = UIntIndex::<u16>::default();
            i.insert(2, 4);

            assert_eq!(*i.eq(2), [4]);
            assert_eq!(3, i.data.len());
        }

        #[test]
        fn or_find_idx_3_4() {
            let mut idx = UIntIndex::new();
            idx.insert(2, 4);
            idx.insert(4, 8);
            idx.insert(3, 6);

            let r = query(idx.eq(3)).or(idx.eq(4)).exec();
            assert_eq!(*r, [6, 8]);

            let q = query(idx.eq(3));
            let r = q.and(idx.eq(3)).exec();
            assert_eq!(*r, [6]);

            let r = query(idx.eq(3)).or(idx.eq(99)).exec();
            assert_eq!(*r, [6]);

            let r = query(idx.eq(99)).or(idx.eq(4)).exec();
            assert_eq!(*r, [8]);

            let r = query(idx.eq(3)).and(idx.eq(4)).exec();
            assert_eq!(&*r, EMPTY_IDXS);

            idx.insert(99, 0);
            let r = query(idx.eq(99)).exec();
            assert_eq!(*r, [0]);
        }

        #[test]
        fn query_and_or() {
            let mut idx = UIntIndex::<usize>::default();
            idx.insert(2, 4);
            idx.insert(4, 8);
            idx.insert(3, 6);

            let r = query(idx.eq(3)).and(idx.eq(2)).exec();
            assert_eq!(&*r, EMPTY_IDXS);

            let r = query(idx.eq(3)).or(idx.eq(4)).and(idx.eq(2)).exec();
            // =3 or =4 and =2 =>
            // (
            // (4 and 2 = false) // `and` has higher prio than `or`
            //  or 3 = true
            // )
            // => 3 -> 6
            assert_eq!(*r, [6]);
        }

        #[test]
        fn out_of_bound() {
            let i = UIntIndex::<u8>::default();
            assert_eq!(0, i.eq(2).len());
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
            let mut i = UIntIndex::<u8>::default();
            i.insert(5, 5);
            i.insert(2, 2);
            i.insert(6, 6);

            assert_eq!(0, i.eq_iter([]).iter().len());
            assert_eq!(0, i.eq_iter([9]).iter().len());
            assert_eq!([2], *i.eq_iter([2]));
            assert_eq!([2, 6], *i.eq_iter([6, 2]));
            assert_eq!([2, 6], *i.eq_iter([9, 6, 2]));
            assert_eq!([2, 5, 6], *i.eq_iter([5, 9, 6, 2]));

            assert_eq!([2, 5, 6], *i.eq_iter(2..=6));
            assert_eq!([2, 5, 6], *i.eq_iter(2..9));
        }

        #[test]
        fn contains() {
            let mut i = UIntIndex::<u8>::default();
            i.insert(5, 5);
            i.insert(2, 2);

            assert!(i.contains(5));
            assert!(!i.contains(55));
        }

        #[test]
        fn min() {
            let mut idx = UIntIndex::<u16>::with_capacity(100);
            assert_eq!(0, idx.min());
            assert_eq!(0, idx._find_min());

            idx.insert(4, 4);
            assert_eq!(4, idx.min());
            assert_eq!(4, idx._find_min());

            idx.insert(2, 8);
            assert_eq!(2, idx.min());
            assert_eq!(2, idx._find_min());

            idx.insert(99, 6);
            assert_eq!(2, idx.min());
            assert_eq!(2, idx._find_min());
        }

        #[test]
        fn min_rm() {
            let mut idx = UIntIndex::<u16>::with_capacity(100);
            idx.insert(4, 4);
            assert_eq!(4, idx.min());
            assert_eq!(4, idx._find_min());

            idx.insert(2, 8);
            assert_eq!(2, idx.min());
            assert_eq!(2, idx._find_min());

            // remove min value on Index 2
            *idx.data.get_mut(2).unwrap() = None;
            assert_eq!(2, idx.min()); // this cached value is now false
            assert_eq!(4, idx._find_min()); // this is the correct value
        }

        #[test]
        fn max() {
            let mut idx = UIntIndex::<u16>::with_capacity(100);
            assert_eq!(0, idx.max());

            idx.insert(4, 4);
            assert_eq!(4, idx.max());

            idx.insert(2, 8);
            assert_eq!(4, idx.max());

            idx.insert(99, 6);
            assert_eq!(99, idx.max());
        }
    }

    mod multi {
        use super::*;

        #[test]
        fn empty() {
            let i = UIntIndex::<u8>::default();
            assert_eq!(0, i.eq(2).len());
            assert!(i.data.is_empty());
        }

        #[test]
        fn find_idx_2() {
            let mut i = UIntIndex::<u8>::default();
            i.insert(2, 2);

            assert_eq!(*i.eq(2), [2]);
            assert_eq!(3, i.data.len());
        }

        #[test]
        fn double_index() {
            let mut i = UIntIndex::<u8>::default();
            i.insert(2, 2);
            i.insert(2, 1);

            assert_eq!(*i.eq(2), [1, 2]);
        }

        #[test]
        fn find_eq_many_unique() {
            let mut i = UIntIndex::<u8>::default();
            i.insert(5, 5);
            i.insert(2, 2);
            i.insert(2, 1);
            i.insert(6, 6);

            assert_eq!(0, i.eq_iter([]).iter().len());
            assert_eq!(0, i.eq_iter([9]).iter().len());
            assert_eq!([1, 2], *i.eq_iter([2]));
            assert_eq!([1, 2, 6], *i.eq_iter([6, 2]));
            assert_eq!([1, 2, 6], *i.eq_iter([9, 6, 2]));
            assert_eq!([1, 2, 5, 6], *i.eq_iter([5, 9, 6, 2]));
        }

        #[test]
        fn contains() {
            let mut i = UIntIndex::<u8>::default();
            i.insert(2, 2);
            i.insert(2, 1);

            assert!(i.contains(2));
            assert!(!i.contains(55));
        }
    }
}
