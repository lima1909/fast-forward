//! Index for 32-bit unsigned integer type.

use std::ops::Deref;

use crate::ops;

use super::{Filter, Idx, IdxFilter, Index, KeyIdxStore, Result};

/// Index for 32-bit unsigned integer type [`usize`].
///
/// Well suitable for for example Primary Keys
///
///```java
/// let _unique_values = vec![3, 2, 4, 1, ...];
///
/// Unique Index:
///
///  Key | Idx (_values)
/// --------------------
///  0   |  -
///  1   |  3
///  2   |  1
///  3   |  0
///  4   |  2
/// ...  | ...
///
/// ```

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
    use crate::ops::{eq, EQ};

    mod unique {

        use std::collections::HashSet;

        use crate::index::{IndexError, Unique};
        use crate::query::{IdxFilterQuery, Query};

        use super::*;

        #[test]
        fn empty() {
            let i = UIntVecIndex::<Unique>::default();
            assert_eq!(0, i.idx(Filter::new(EQ, 2)).len());
            assert!(i.0.is_empty());
        }

        #[test]
        fn find_idx_2() {
            let mut i = UIntVecIndex::<Unique>::default();
            i.insert(2, 4).unwrap();

            assert_eq!(i.idx(Filter::new(EQ, 2)), &[4]);
            assert_eq!(3, i.0.len());
        }

        #[test]
        fn or_find_idx_3_4() {
            let mut i = UIntVecIndex::<Unique>::default();
            i.insert(2, 4).unwrap();
            i.insert(4, 8).unwrap();
            i.insert(3, 6).unwrap();

            let mut q = IdxFilterQuery::new(i, HashSet::default());
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
            let mut i = UIntVecIndex::<Unique>::default();
            i.insert(2, 2).unwrap();

            assert_eq!(Err(IndexError::NotUniqueKey), i.insert(2, 2));
        }

        #[test]
        fn out_of_bound() {
            let i = UIntVecIndex::<Unique>::default();
            assert_eq!(0, i.idx(Filter::new(EQ, 2)).len());
        }
    }

    mod multi {
        use crate::index::Multi;

        use super::*;

        #[test]
        fn empty() {
            let i = UIntVecIndex::<Multi>::default();
            assert_eq!(0, i.idx(Filter::new(EQ, 2)).len());
            assert!(i.0.is_empty());
        }

        #[test]
        fn find_idx_2() {
            let mut i = UIntVecIndex::<Multi>::default();
            i.insert(2, 2).unwrap();

            assert!(i.idx(Filter::new(EQ, 2)).eq(&[2]));
            assert_eq!(3, i.0.len());
        }

        #[test]
        fn double_index() {
            let mut i = UIntVecIndex::<Multi>::default();
            i.insert(2, 2).unwrap();
            i.insert(2, 1).unwrap();

            assert!(i.idx(Filter::new(EQ, 2)).eq(&[2, 1]));
        }
    }
}
