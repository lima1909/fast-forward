//! Indices for 32-bit unsigned integer type ([`usize`]).
//!
//! Well suitable for for example `Primary Keys`
//!
//!```java
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
//! ```
use super::{Filter, Idx, IdxFilter, Index, KeyIdxStore, Multi, Result, Unique};
use crate::ops;
use std::ops::Deref;

/// Unique `Primary Key` from type [`usize`].
pub type PkUintIdx = UIntVecIndex<Unique>;

/// An not unique Key, which can occur multiple times.
pub type MultiUintIdx = UIntVecIndex<Multi>;

/// `Key` is from type [`crate::Idx`] and the information are saved in a List (Store).
#[derive(Debug, Default)]
pub struct UIntVecIndex<I: Index>(Vec<Option<I>>);

impl<I: Index> IdxFilter<Idx> for UIntVecIndex<I> {
    fn idx(&self, f: Filter<Idx>) -> &[Idx] {
        if f.op != ops::EQ {
            return &[];
        }

        match &self.0.get(f.key) {
            Some(Some(idx)) => idx.get(),
            _ => &[],
        }
    }
}

impl<I: Index + Clone> KeyIdxStore<Idx> for UIntVecIndex<I> {
    fn insert(&mut self, key: Idx, i: Idx) -> Result {
        if self.0.len() <= key {
            self.0.resize(key + 1, None);
        }

        match self.0[key].as_mut() {
            Some(idx) => idx.add(i)?,
            None => self.0[key] = Some(I::new(i)),
        }

        Ok(())
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

    mod unique {
        use super::*;
        use std::collections::HashSet;

        use crate::{
            index::IndexError,
            ops::eq,
            query::{Query, ToQuery},
        };

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

            assert_eq!(i.eq(2), &[4]);
            assert_eq!(i.ne(3), &[]); // TODO: `ne` do not work now
            assert_eq!(3, i.0.len());
        }

        #[test]
        fn or_find_idx_3_4() {
            let mut idx = PkUintIdx::default();
            idx.insert(2, 4).unwrap();
            idx.insert(4, 8).unwrap();
            idx.insert(3, 6).unwrap();

            let mut q = idx.to_query(HashSet::new());
            let r = q.filter(eq("", 3)).or(eq("", 4)).exec();
            assert!(r.contains(&8));
            assert!(r.contains(&6));

            let r = q.reset().filter(eq("", 3)).or(eq("", 99)).exec();
            assert!(r.contains(&6));

            let r = q.reset().filter(eq("", 99)).or(eq("", 4)).exec();
            assert!(r.contains(&8));

            let r = q.reset().filter(eq("", 3)).or(eq("", 4)).exec();
            assert!(r.contains(&8));
            assert!(r.contains(&6));
        }

        #[test]
        fn double_index() {
            let mut i = PkUintIdx::default();
            i.insert(2, 2).unwrap();

            assert_eq!(Err(IndexError::NotUniqueKey), i.insert(2, 2));
        }

        #[test]
        fn out_of_bound() {
            let i = PkUintIdx::default();
            assert_eq!(0, i.eq(2).len());
        }

        #[test]
        fn query_or_without_filter() {
            let mut idx = PkUintIdx::default();
            idx.insert(2, 2).unwrap();

            let mut q = idx.to_query(HashSet::new());
            assert_eq!(vec![2], q.or(eq("", 2)).exec());
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

            assert_eq!(i.eq(2), &[2]);
            assert_eq!(3, i.0.len());
        }

        #[test]
        fn double_index() {
            let mut i = MultiUintIdx::default();
            i.insert(2, 2).unwrap();
            i.insert(2, 1).unwrap();

            assert_eq!(i.eq(2), [2, 1]);
        }
    }
}
