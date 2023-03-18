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
    query::EMPTY_IDXS,
    Key,
};
use std::{borrow::Cow, ops::Deref};

/// Unique `Primary Key` from type [`usize`].
pub type PkUintIdx = UIntVecIndex<Unique>;

/// An not unique Key, which can occur multiple times.
pub type MultiUintIdx = UIntVecIndex<Multi>;

/// `Key` is from type [`crate::Idx`] and the information are saved in a List (Store).
#[derive(Debug, Default)]
pub struct UIntVecIndex<I: Index>(Vec<Option<I>>);

impl<'s, I: Index + Clone> Store<'s> for UIntVecIndex<I> {
    fn insert(&mut self, key: Key<'s>, i: Idx) -> Result {
        let k = key.try_into()?;
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
    fn filter(&self, p: Predicate<'k>) -> Result<Cow<[usize]>> {
        let i: Idx = p.2.try_into()?;

        let idxs = match &self.0.get(i) {
            Some(Some(idx)) => Cow::Borrowed(idx.get()),
            _ => Cow::Borrowed(EMPTY_IDXS),
        };

        Ok(idxs)
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

    mod unique {
        use super::*;

        #[test]
        fn empty() {
            let i = PkUintIdx::default();
            assert_eq!(0, i.eq(2).unwrap().len());
            assert!(i.0.is_empty());
        }

        #[test]
        fn find_idx_2() {
            let mut i = PkUintIdx::default();
            i.insert_idx(2, 4).unwrap();

            assert_eq!(*i.eq(2).unwrap(), [4]);
            // assert_eq!(i.ne(3), &[]);  TODO: `ne` do not work now
            assert_eq!(3, i.0.len());
        }

        #[test]
        fn or_find_idx_3_4() -> Result {
            let mut idx = PkUintIdx::default();
            idx.insert_idx(2, 4).unwrap();
            idx.insert_idx(4, 8).unwrap();
            idx.insert_idx(3, 6).unwrap();

            {
                let r = idx.query(3).or(4).exec()?;
                assert_eq!(*r, [6, 8]);

                // reuse the query without `new`
                let q = idx.query(3);
                let r = q.and(3).exec()?;
                assert_eq!(*r, [6]);

                let r = idx.query(3).or(99).exec()?;
                assert_eq!(*r, [6]);

                let r = idx.query(99).or(4).exec()?;
                assert_eq!(*r, [8]);

                let r = idx.query(3).and(4).exec()?;
                assert_eq!(*r, []);
            }

            // add a new index after creating a QueryBuilder
            idx.insert_idx(99, 0).unwrap();
            let r = idx.query(99).exec()?;
            assert_eq!(*r, [0]);

            Ok(())
        }

        #[test]
        fn query_and_or() -> Result {
            let mut idx = PkUintIdx::default();
            idx.insert_idx(2, 4).unwrap();
            idx.insert_idx(4, 8).unwrap();
            idx.insert_idx(3, 6).unwrap();

            let r = idx.query(3).and(2).exec()?;
            assert_eq!(*r, []);

            let r = idx.query(3).or(4).and(2).exec()?;
            // =3 or =4 and =2 =>
            // (
            // (4 and 2 = false) // `and` has higher prio than `or`
            //  or 3 = true
            // )
            // => 3 -> 6
            assert_eq!(*r, [6]);

            Ok(())
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
            assert_eq!(0, i.eq(2).unwrap().len());
        }

        #[test]
        fn filter_ivalid_key_type() {
            let i = PkUintIdx::default();
            let err = i.eq("2");
            assert!(err.is_err());
            assert_eq!(
                Error::InvalidKeyType {
                    expected: "usize",
                    got: "str: 2".to_string()
                },
                err.err().unwrap()
            );
        }

        #[test]
        fn query_ivalid_key_type() {
            let i = PkUintIdx::default();
            let err = i.query("2").or(2).exec();
            assert!(err.is_err());
            assert_eq!(
                Error::InvalidKeyType {
                    expected: "usize",
                    got: "str: 2".to_string()
                },
                err.err().unwrap()
            );
        }

        #[test]
        fn insert_invalid_key_type() {
            let mut i = PkUintIdx::default();
            let err = i.insert(Key::Str("false"), 4);
            assert!(err.is_err());
        }

        #[test]
        fn with_capacity() {
            let mut i = PkUintIdx::with_capacity(5);
            i.insert_idx(1, 4).unwrap();
            assert_eq!(2, i.len());
            assert_eq!(5, i.capacity());
        }
    }

    mod multi {
        use super::*;

        #[test]
        fn empty() {
            let i = MultiUintIdx::default();
            assert_eq!(0, i.eq(2).unwrap().len());
            assert!(i.0.is_empty());
        }

        #[test]
        fn find_idx_2() {
            let mut i = MultiUintIdx::default();
            i.insert_idx(2, 2).unwrap();

            assert_eq!(*i.eq(2).unwrap(), [2]);
            assert_eq!(3, i.0.len());
        }

        #[test]
        fn double_index() {
            let mut i = MultiUintIdx::default();
            i.insert_idx(2, 2).unwrap();
            i.insert_idx(2, 1).unwrap();

            assert_eq!(*i.eq(2).unwrap(), [1, 2]);
        }
    }
}
