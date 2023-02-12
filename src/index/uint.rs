use std::ops::Index;

use super::{AmbiguousIndex, AsSlice, Idx, IndexError, Key, Result, Store, UniqueIndex};

/// Index for
///
/// Well suitable for `unsigned integer (u32)` ( for example Primary Keys).
///
///```java
/// let _primary_keys = vec![1, 2, 3, ...];
///
/// PrimaryKey | Position
/// ----------------------
///     0      |   -
///     1      |   0
///     2      |   1
///     3      |   2
///    ...     |  ...
/// ```
#[derive(Debug, Default)]
pub struct U32Index<I>(I);

impl<I: ListIndex> U32Index<I> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<I: ListIndex> Index<(Key, &'static str)> for U32Index<I> {
    type Output = [Idx];

    fn index(&self, key: (Key, &'static str)) -> &Self::Output {
        if key.1 != "=" {
            todo!()
        }

        let idx = match key.0 {
            Key::Number(super::Number::Usize(u)) => u,
            Key::Number(super::Number::I32(i)) => usize::try_from(i).ok().unwrap(),
            _ => todo!(),
        };

        self.0.as_slice(idx)
    }
}

impl<I: ListIndex> Store for U32Index<I> {
    fn insert(&mut self, key: &Key, idx: Idx) -> Result {
        let i = match key {
            Key::Number(super::Number::Usize(u)) => *u,
            Key::Number(super::Number::I32(i)) => usize::try_from(*i).ok().unwrap(),
            _ => todo!(),
        };
        self.0.insert(i, idx)
    }
}

pub trait ListIndex: Default {
    fn insert(&mut self, key: Idx, idx: Idx) -> Result;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn as_slice(&self, i: Idx) -> &[Idx];
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Unique(Vec<Option<UniqueIndex>>);

impl ListIndex for Unique {
    fn insert(&mut self, key: Idx, idx: Idx) -> Result {
        if self.0.len() <= key {
            self.0.resize(key + 1, None);
        }

        if self.0[key].is_some() {
            return Err(IndexError::NotUnique(key.into()));
        }

        self.0[key] = Some(idx.into());
        Ok(())
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn as_slice(&self, i: Idx) -> &[Idx] {
        match self.0.get(i) {
            Some(Some(i)) => i.as_slice(),
            _ => &[],
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Ambiguous(Vec<Option<AmbiguousIndex>>);

impl ListIndex for Ambiguous {
    fn insert(&mut self, key: Idx, idx: Idx) -> Result {
        if self.0.len() <= key {
            self.0.resize(key + 1, None);
        }

        match self.0[key].as_mut() {
            Some(i) => i.push(idx),
            None => self.0[key] = Some(AmbiguousIndex::new(idx)),
        }

        Ok(())
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn as_slice(&self, i: Idx) -> &[Idx] {
        match self.0.get(i) {
            Some(Some(i)) => i.as_slice(),
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
                Err(IndexError::NotUnique(2usize.into())),
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
