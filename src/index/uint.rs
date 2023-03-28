//! Indices for 32-bit unsigned integer type ([`usize`]).
//!
//! Well suitable for for example `Primary Keys`.
//!
//! The `Key` is the position (index) in the Index-Vec ([`UIntVecIndex`]).
//!
//!```text
//! let _unique_values = vec![3, 2, 4, 1, ...];
//!
//! Unique Index:
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
//! let _multi_values = vec![3, 2, 3, 1, 2, 2, ...];
//!
//! Muli Index:
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
use std::borrow::Cow;

use super::Equals;

/// Short name for [`UIntVecIndex`].
pub type UIntIndex = UIntVecIndex;

/// `Key` is from type [`crate::Idx`] and the information are saved in a List (Store).
#[derive(Debug, Default)]
pub struct UIntVecIndex(Vec<Option<Index>>);

impl Store<Idx> for UIntVecIndex {
    fn insert(&mut self, k: Idx, i: Idx) {
        if self.0.len() <= k {
            self.0.resize(k + 1, None);
        }

        match self.0[k].as_mut() {
            Some(idx) => idx.add(i),
            None => self.0[k] = Some(Index::new(i)),
        }
    }
}

impl Equals<usize> for UIntVecIndex {
    #[inline]
    fn eq(&self, key: usize) -> Cow<[Idx]> {
        match &self.0.get(key) {
            Some(Some(idx)) => idx.get(),
            _ => Cow::Borrowed(EMPTY_IDXS),
        }
    }
}

impl UIntVecIndex {
    pub fn with_capacity(capacity: usize) -> Self {
        UIntVecIndex(Vec::with_capacity(capacity))
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
            let i = UIntIndex::default();
            assert_eq!(0, i.eq(2).len());
            assert!(i.0.is_empty());
        }

        #[test]
        fn find_idx_2() {
            let mut i = UIntIndex::default();
            i.insert(2, 4);

            assert_eq!(*i.eq(2), [4]);
            // assert_eq!(i.ne(3), &[]);  TODO: `ne` do not work now
            assert_eq!(3, i.0.len());
        }

        #[test]
        fn or_find_idx_3_4() {
            let mut idx = UIntIndex::default();
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
            let mut idx = UIntIndex::default();
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
            let i = UIntIndex::default();
            assert_eq!(0, i.eq(2).len());
        }

        #[test]
        fn with_capacity() {
            let mut i = UIntIndex::with_capacity(5);
            i.insert(1, 4);
            assert_eq!(2, i.0.len());
            assert_eq!(5, i.0.capacity());
        }

        #[test]
        fn find_eq_many_unique() {
            let mut i = UIntIndex::default();
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
    }

    mod multi {
        use super::*;

        #[test]
        fn empty() {
            let i = UIntIndex::default();
            assert_eq!(0, i.eq(2).len());
            assert!(i.0.is_empty());
        }

        #[test]
        fn find_idx_2() {
            let mut i = UIntIndex::default();
            i.insert(2, 2);

            assert_eq!(*i.eq(2), [2]);
            assert_eq!(3, i.0.len());
        }

        #[test]
        fn double_index() {
            let mut i = UIntIndex::default();
            i.insert(2, 2);
            i.insert(2, 1);

            assert_eq!(*i.eq(2), [1, 2]);
        }

        #[test]
        fn find_eq_many_unique() {
            let mut i = UIntIndex::default();
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
    }
}
