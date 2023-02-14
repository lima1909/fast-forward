//! Index for 32-bit unsigned integer type.
use std::ops::{Deref, DerefMut, Index};

use crate::{ops, Filter};

use super::{Idx, Key, Result, Store, UniformIdx};

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
pub struct U32Index<I: UniformIdx>(ListIndex<I>);

impl<I: UniformIdx> Index<Filter> for U32Index<I> {
    type Output = [Idx];

    fn index(&self, f: Filter) -> &Self::Output {
        match f.key().get_usize() {
            Ok(idx) => match f.op() {
                ops::EQ => self.0.as_idx_slice(idx),
                _ => &[],
            },
            Err(_) => &[],
        }
    }
}

impl<I: UniformIdx + Clone> Store for U32Index<I> {
    fn insert(&mut self, key: &Key, idx: Idx) -> Result {
        self.0.insert_idx(key.get_usize()?, idx)
    }
}
#[derive(Debug, Default)]
struct ListIndex<I: UniformIdx>(Vec<Option<I>>);

impl<I: UniformIdx> ListIndex<I> {
    fn insert_idx(&mut self, key: Idx, idx: Idx) -> Result
    where
        I: Clone,
    {
        if self.len() <= key {
            self.resize(key + 1, None);
        }

        match self[key].as_mut() {
            Some(i) => i.add(idx),
            None => {
                self[key] = Some(I::new(idx));
                Ok(())
            }
        }
    }

    fn as_idx_slice(&self, i: Idx) -> &[Idx] {
        match self.get(i) {
            Some(Some(i)) => i.as_slice(),
            _ => &[],
        }
    }
}

impl<I: UniformIdx> Deref for ListIndex<I> {
    type Target = Vec<Option<I>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<I: UniformIdx> DerefMut for ListIndex<I> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod unique {
        use super::*;
        use crate::{
            index::{IndexError, UniqueIdx},
            ops::eq,
            Query,
        };

        #[test]
        fn empty() {
            let idx = U32Index::<UniqueIdx>::default();
            assert_eq!(0, idx[eq(2usize)].len());
            assert!(idx.0.is_empty());
        }

        #[test]
        fn find_idx_2() {
            let mut idx = U32Index::<UniqueIdx>::default();
            idx.insert(&2.into(), 4).unwrap();

            assert_eq!(idx.filter(eq(2)), &[4]);
            assert_eq!(3, idx.0.len());
        }

        #[test]
        fn or_find_idx_3_4() {
            let mut idx = U32Index::<UniqueIdx>::default();
            idx.insert(&2.into(), 4).unwrap();
            idx.insert(&4.into(), 8).unwrap();
            idx.insert(&3.into(), 6).unwrap();

            let r = idx.or(eq(3), eq(4));
            assert!(r.contains(&&8));
            assert!(r.contains(&&6));

            let r = idx.or(eq(3), eq(99));
            assert!(r.contains(&&6));

            let r = idx.or(eq(99), eq(4));
            assert!(r.contains(&&8));
        }

        #[test]
        fn double_index() {
            let mut idx = U32Index::<UniqueIdx>::default();
            idx.insert(&2.into(), 2).unwrap();

            assert_eq!(
                Err(IndexError::NotUniqueKey(2usize.into())),
                idx.insert(&2.into(), 2)
            );
        }

        #[test]
        fn out_of_bound() {
            let idx = U32Index::<UniqueIdx>::default();
            assert_eq!(0, idx[eq(2)].len());
        }
    }

    mod ambiguous {
        use super::*;
        use crate::{index::AmbiguousIdx, ops::eq};

        #[test]
        fn empty() {
            let idx = U32Index::<AmbiguousIdx>::default();
            assert_eq!(0, idx[eq(2)].len());
            assert!(idx.0.is_empty());
        }

        #[test]
        fn find_idx_2() {
            let mut idx = U32Index::<AmbiguousIdx>::default();
            idx.insert(&2.into(), 2).unwrap();

            assert!(idx[eq(2)].eq(&[2]));
            assert_eq!(3, idx.0.len());
        }

        #[test]
        fn double_index() {
            let mut idx = U32Index::<AmbiguousIdx>::default();
            idx.insert(&2.into(), 2).unwrap();
            idx.insert(&2.into(), 1).unwrap();

            assert!(idx[eq(2)].eq(&[2, 1]));
        }
    }
}
