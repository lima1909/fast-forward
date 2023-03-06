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
    index::{Filterable, Idx, Index, Multi, Predicate, Result, Store, Unique},
    Key,
};
use std::ops::Deref;

/// Unique `Primary Key` from type [`usize`].
pub type PkUintIdx = UIntVecIndex<Unique>;

/// An not unique Key, which can occur multiple times.
pub type MultiUintIdx = UIntVecIndex<Multi>;

/// `Key` is from type [`crate::Idx`] and the information are saved in a List (Store).
#[derive(Debug, Default)]
pub struct UIntVecIndex<I: Index>(Vec<Option<I>>);

impl<'k, I: Index + Clone> Store<'k> for UIntVecIndex<I> {
    fn insert(&mut self, key: Key<'k>, i: Idx) -> Result {
        let k = key.into();
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

impl<'k, I: Index> Filterable<'k> for UIntVecIndex<I> {
    fn filter(&self, p: Predicate<'k>) -> &[Idx] {
        let i: Idx = p.2.into();
        match &self.0.get(i) {
            Some(Some(idx)) => idx.get(),
            _ => &[],
        }
    }
}
impl<I: Index + Clone> UIntVecIndex<I> {
    pub fn insert_idx(&mut self, idx: Idx, i: Idx) -> Result {
        self.insert(idx.into(), i)
    }
}

impl<I: Index> UIntVecIndex<I> {
    pub fn with_capacity(capacity: usize) -> Self {
        UIntVecIndex(Vec::with_capacity(capacity))
    }
}

impl<I: Index> Deref for UIntVecIndex<I> {
    type Target = Vec<Option<I>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::Queryable;
    use crate::{error::Error, index::OpsFilter};
    use std::collections::HashSet;

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
            i.insert_idx(2, 4).unwrap();

            assert_eq!(i.eq(2), &[4]);
            // assert_eq!(i.ne(3), &[]);  TODO: `ne` do not work now
            assert_eq!(3, i.0.len());
        }

        #[test]
        fn or_find_idx_3_4() {
            let mut idx = PkUintIdx::default();
            idx.insert_idx(2, 4).unwrap();
            idx.insert_idx(4, 8).unwrap();
            idx.insert_idx(3, 6).unwrap();

            {
                let b = idx.query_builder::<HashSet<Idx>>();
                let r: Vec<Idx> = b.query(3).or(4).exec().collect();
                assert!(r.contains(&8));
                assert!(r.contains(&6));

                // reuse the query without `new`
                let q = b.query(3);
                let r = q.and(3).exec().next();
                assert_eq!(Some(6), r);

                let r = b.query(3).or(99).exec().next();
                assert_eq!(r, Some(6));

                let r = b.query(99).or(4).exec().next();
                assert_eq!(r, Some(8));

                let r = b.query(3).and(4).exec().next();
                assert_eq!(r, None);
            }

            // add a new index after creating a QueryBuilder
            idx.insert_idx(99, 0).unwrap();
            let b = idx.query_builder::<HashSet<Idx>>();
            let mut r = b.query(99).exec();
            assert_eq!(r.next(), Some(0));
        }

        #[test]
        fn query_and_or() {
            let mut idx = PkUintIdx::default();
            idx.insert_idx(2, 4).unwrap();
            idx.insert_idx(4, 8).unwrap();
            idx.insert_idx(3, 6).unwrap();

            let b = idx.query_builder::<HashSet<Idx>>();
            let mut r = b.query(3).and(2).exec();
            assert_eq!(r.next(), None);

            let mut r = b.query(3).or(4).and(2).exec();
            // =3 or =4 and =2 =>
            // (
            // (4 and 2 = false) // `and` has higher prio than `or`
            //  or 3 = true
            // )
            // => 3 -> 6
            assert_eq!(Some(6), r.next());
        }

        #[test]
        fn double_index() {
            let mut i = PkUintIdx::default();
            i.insert_idx(2, 2).unwrap();

            assert_eq!(Err(Error::NotUniqueIndexKey), i.insert_idx(2, 2));
        }

        #[test]
        fn out_of_bound() {
            let i = PkUintIdx::default();
            assert_eq!(0, i.eq(2).len());
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
            i.insert_idx(2, 2).unwrap();

            assert_eq!(i.eq(2), &[2]);
            assert_eq!(3, i.0.len());
        }

        #[test]
        fn double_index() {
            let mut i = MultiUintIdx::default();
            i.insert_idx(2, 2).unwrap();
            i.insert_idx(2, 1).unwrap();

            assert_eq!(i.eq(2), [2, 1]);
        }
    }
}
