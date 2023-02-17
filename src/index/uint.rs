//! Index for 32-bit unsigned integer type.

use std::ops::Deref;

use crate::{ops, Filter};

use super::{Idx, IdxFilter, IndexError, KeyIdxStore, Result};

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

pub trait Index {
    fn new(i: Idx) -> Self;
    fn add(&mut self, i: Idx) -> Result;
    fn get(&self) -> &[Idx];
}

#[derive(Debug, Default, Clone)]
pub struct Unique([Idx; 1]);

impl Index for Unique {
    #[inline]
    fn new(i: Idx) -> Self {
        Unique([i])
    }

    #[inline]
    fn add(&mut self, _i: Idx) -> Result {
        Err(IndexError::NotUniqueKey)
    }

    #[inline]
    fn get(&self) -> &[Idx] {
        &self.0
    }
}

#[derive(Debug, Default, Clone)]
pub struct Multi(Vec<Idx>);

impl Index for Multi {
    #[inline]
    fn new(i: Idx) -> Self {
        Multi(vec![i])
    }

    #[inline]
    fn add(&mut self, i: Idx) -> Result {
        self.0.push(i);
        Ok(())
    }

    #[inline]
    fn get(&self) -> &[Idx] {
        &self.0
    }
}

/// `Key` is from type [`crate::Idx`] and the information are saved in a List (Store).
#[derive(Debug, Default)]
pub struct UIntVecIndex<I: Index>(Vec<Option<I>>);

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

impl<I: Index> IdxFilter<Idx> for UIntVecIndex<I> {
    fn idx(&self, f: Filter<Idx>) -> &[Idx] {
        if f.0 != ops::EQ {
            return &[];
        }

        match &self.0.get(f.1) {
            Some(Some(idx)) => idx.get(),
            _ => &[],
        }
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
    use crate::{ops::eq, Query};

    mod unique {
        use super::*;

        #[test]
        fn empty() {
            let i = UIntVecIndex::<Unique>::default();
            assert_eq!(0, i.idx(eq(2)).len());
            assert!(i.0.is_empty());
        }

        #[test]
        fn find_idx_2() {
            let mut i = UIntVecIndex::<Unique>::default();
            i.insert(2, 4).unwrap();

            assert_eq!(i.idx(eq(2)), &[4]);
            assert_eq!(3, i.0.len());
        }

        #[test]
        fn or_find_idx_3_4() {
            let mut i = UIntVecIndex::<Unique>::default();
            i.insert(2, 4).unwrap();
            i.insert(4, 8).unwrap();
            i.insert(3, 6).unwrap();

            let r = i.or(eq(3), eq(4));
            assert!(r.contains(&&8));
            assert!(r.contains(&&6));

            let r = i.or(eq(3), eq(99));
            assert!(r.contains(&&6));

            let r = i.or(eq(99), eq(4));
            assert!(r.contains(&&8));
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
            assert_eq!(0, i.filter(eq(2)).len());
        }
    }

    mod multi {
        use super::*;

        #[test]
        fn empty() {
            let i = UIntVecIndex::<Multi>::default();
            assert_eq!(0, i.idx(eq(2)).len());
            assert!(i.0.is_empty());
        }

        #[test]
        fn find_idx_2() {
            let mut i = UIntVecIndex::<Multi>::default();
            i.insert(2, 2).unwrap();

            assert!(i.idx(eq(2)).eq(&[2]));
            assert_eq!(3, i.0.len());
        }

        #[test]
        fn double_index() {
            let mut i = UIntVecIndex::<Multi>::default();
            i.insert(2, 2).unwrap();
            i.insert(2, 1).unwrap();

            assert!(i.filter(eq(2)).eq(&[2, 1]));
        }
    }
}
