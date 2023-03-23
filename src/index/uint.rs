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
    index::{Idx, Index, Multi, Result, Store, Unique},
    query::EMPTY_IDXS,
};
use std::borrow::Cow;

/// Unique `Primary Key` from type [`usize`].
pub type PkUintIdx = UIntVecIndex<Unique>;

/// An not unique Key, which can occur multiple times.
pub type MultiUintIdx = UIntVecIndex<Multi>;

/// `Key` is from type [`crate::Idx`] and the information are saved in a List (Store).
#[derive(Debug, Default)]
pub struct UIntVecIndex<I: Index>(Vec<Option<I>>);

impl<I> Store<Idx> for UIntVecIndex<I>
where
    I: Index + Clone,
{
    fn insert(&mut self, k: Idx, i: Idx) -> Result {
        if self.0.len() <= k {
            self.0.resize(k + 1, None);
        }

        match self.0[k].as_mut() {
            Some(idx) => idx.add(i)?,
            None => self.0[k] = Some(I::new(i)),
        }

        Ok(())
    }
}

impl<I> UIntVecIndex<I>
where
    I: Index,
{
    pub fn eq(&self, i: usize) -> Cow<[Idx]> {
        match &self.0.get(i) {
            Some(Some(idx)) => Cow::Borrowed(idx.get()),
            _ => Cow::Borrowed(EMPTY_IDXS),
        }
    }
}

impl<I> UIntVecIndex<I>
where
    I: Index + Clone,
{
    pub fn with_capacity(capacity: usize) -> Self {
        UIntVecIndex(Vec::with_capacity(capacity))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use crate::query;

    mod unique {
        use super::*;

        #[test]
        fn empty() {
            let i = PkUintIdx::default();
            assert_eq!(0, i.eq(2).len());
            assert!(i.0.is_empty());
        }

        #[test]
        fn find_idx_2() {
            let mut i = PkUintIdx::default();
            i.insert(2, 4).unwrap();

            assert_eq!(*i.eq(2), [4]);
            // assert_eq!(i.ne(3), &[]);  TODO: `ne` do not work now
            assert_eq!(3, i.0.len());
        }

        #[test]
        fn or_find_idx_3_4() {
            let mut idx = PkUintIdx::default();
            idx.insert(2, 4).unwrap();
            idx.insert(4, 8).unwrap();
            idx.insert(3, 6).unwrap();

            let r = query(idx.eq(3)).or(idx.eq(4)).exec();
            assert_eq!(*r, [6, 8]);

            // reuse the query without `new`
            let q = query(idx.eq(3));
            let r = q.and(idx.eq(3)).exec();
            assert_eq!(*r, [6]);

            let r = query(idx.eq(3)).or(idx.eq(99)).exec();
            assert_eq!(*r, [6]);

            let r = query(idx.eq(99)).or(idx.eq(4)).exec();
            assert_eq!(*r, [8]);

            let r = query(idx.eq(3)).and(idx.eq(4)).exec();
            assert_eq!(&*r, EMPTY_IDXS);

            // add a new index after creating a QueryBuilder
            idx.insert(99, 0).unwrap();
            let r = query(idx.eq(99)).exec();
            assert_eq!(*r, [0]);
        }

        #[test]
        fn query_and_or() {
            let mut idx = PkUintIdx::default();
            idx.insert(2, 4).unwrap();
            idx.insert(4, 8).unwrap();
            idx.insert(3, 6).unwrap();

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
        fn double_index() {
            let mut i = PkUintIdx::default();
            i.insert(2, 2).unwrap();

            assert_eq!(Err(Error::NotUniqueIndexKey), i.insert(2, 2));
        }

        #[test]
        fn out_of_bound() {
            let i = PkUintIdx::default();
            assert_eq!(0, i.eq(2).len());
        }

        #[test]
        fn with_capacity() {
            let mut i = PkUintIdx::with_capacity(5);
            i.insert(1, 4).unwrap();
            assert_eq!(2, i.0.len());
            assert_eq!(5, i.0.capacity());
        }
    }

    mod multi {
        use super::*;

        #[test]
        fn empty() {
            let i = MultiUintIdx::default();
            assert_eq!(0, i.eq(2).len());
            assert!(i.0.is_empty());
        }

        #[test]
        fn find_idx_2() {
            let mut i = MultiUintIdx::default();
            i.insert(2, 2).unwrap();

            assert_eq!(*i.eq(2), [2]);
            assert_eq!(3, i.0.len());
        }

        #[test]
        fn double_index() {
            let mut i = MultiUintIdx::default();
            i.insert(2, 2).unwrap();
            i.insert(2, 1).unwrap();

            assert_eq!(*i.eq(2), [1, 2]);
        }
    }
}
