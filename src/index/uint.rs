//! Index for 32-bit unsigned integer type.
use std::ops::{Deref, Index};

use super::{AmbiguousIdx, AsIdxSlice, Idx, IndexError, Key, Result, Store, UniqueIdx};

/// Index for 32-bit unsigned integer type [`u32`].
///
/// Well suitable for for example Primary Keys
///```java
/// let _unique_values = vec![3, 2, 4, 1, ...];
///
/// UniqueIdx:
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
#[derive(Debug, Default)]
pub struct U32Index<I>(I);

impl<I: ListIndex> U32Index<I> {}

impl<I> Deref for U32Index<I> {
    type Target = I;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<I: ListIndex> Index<(Key, &'static str)> for U32Index<I> {
    type Output = [Idx];

    fn index(&self, key: (Key, &'static str)) -> &Self::Output {
        if key.1 != "=" {
            todo!()
        }

        match key.0.get_usize() {
            Ok(idx) => self.as_slice(idx),
            Err(_) => &[],
        }
    }
}

impl<I: ListIndex> Store for U32Index<I> {
    fn insert(&mut self, key: &Key, idx: Idx) -> Result {
        self.0.insert(key.get_usize()?, idx)
    }
}

#[allow(private_in_public)]
trait ListIndex: Default {
    fn insert(&mut self, key: Idx, idx: Idx) -> Result;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn as_slice(&self, i: Idx) -> &[Idx];
}

/// Unique Index.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Unique(Vec<Option<UniqueIdx>>);

impl ListIndex for Unique {
    fn insert(&mut self, key: Idx, idx: Idx) -> Result {
        if self.len() <= key {
            self.0.resize(key + 1, None);
        }

        if self.0[key].is_some() {
            return Err(IndexError::NotUniqueKey(key.into()));
        }

        self.0[key] = Some(idx.into());
        Ok(())
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn as_slice(&self, i: Idx) -> &[Idx] {
        match self.0.get(i) {
            Some(Some(i)) => i.as_idx_slice(),
            _ => &[],
        }
    }
}

/// Ambiguous Index.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Ambiguous(Vec<Option<AmbiguousIdx>>);

impl ListIndex for Ambiguous {
    fn insert(&mut self, key: Idx, idx: Idx) -> Result {
        if self.0.len() <= key {
            self.0.resize(key + 1, None);
        }

        match self.0[key].as_mut() {
            Some(i) => i.push(idx),
            None => self.0[key] = Some(AmbiguousIdx::new(idx)),
        }

        Ok(())
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn as_slice(&self, i: Idx) -> &[Idx] {
        match self.0.get(i) {
            Some(Some(i)) => i.as_idx_slice(),
            _ => &[],
        }
    }
}

#[cfg(test)]
mod tests {

    mod unique {
        use super::super::*;

        #[test]
        fn empty() {
            let idx = U32Index::<Unique>::default();
            assert_eq!(0, idx.index((2.into(), "=")).len());
            assert!(idx.is_empty());
        }

        #[test]
        fn find_idx_2() {
            let mut idx = U32Index::<Unique>::default();
            idx.insert(&2.into(), 2).unwrap();

            assert!(idx[(2.into(), "=")].eq(&[2]));
            assert_eq!(3, idx.len());
        }

        #[test]
        fn double_index() {
            let mut idx = U32Index::<Unique>::default();
            idx.insert(&2.into(), 2).unwrap();

            assert_eq!(
                Err(IndexError::NotUniqueKey(2usize.into())),
                idx.insert(&2.into(), 2)
            );
        }

        #[test]
        fn out_of_bound() {
            let idx = U32Index::<Unique>::default();
            assert_eq!(0, idx.index((2.into(), "=")).len());
        }
    }

    mod ambiguous {
        use super::super::*;

        #[test]
        fn empty() {
            let idx = U32Index::<Ambiguous>::default();
            assert_eq!(0, idx.index((2.into(), "=")).len());
            assert!(idx.is_empty());
        }

        #[test]
        fn find_idx_2() {
            let mut idx = U32Index::<Ambiguous>::default();
            idx.insert(&2.into(), 2).unwrap();

            assert!(idx[(2.into(), "=")].eq(&[2]));
            assert_eq!(3, idx.len());
        }

        #[test]
        fn double_index() {
            let mut idx = U32Index::<Ambiguous>::default();
            idx.insert(&2.into(), 2).unwrap();
            idx.insert(&2.into(), 1).unwrap();

            assert!(idx[(2.into(), "=")].eq(&[2, 1]));
        }
    }
}
