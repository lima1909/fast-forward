//! Index for 32-bit unsigned integer type.

use crate::{ops, Filter};

use super::{Idx, IndexError, Indexer, Key, KeyIdxStore, Result};

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
pub struct UniqueUsizeIndex(Vec<Option<[Idx; 1]>>);

impl KeyIdxStore<Idx> for UniqueUsizeIndex {
    fn insert(&mut self, key: Idx, i: Idx) -> Result {
        if self.0.len() <= key {
            self.0.resize(key + 1, None);
        }

        match self.0[key].as_mut() {
            Some(_i) => Err(IndexError::NotUniqueKey(Key::Usize(key))),
            None => {
                self.0[key] = Some([i]);
                Ok(())
            }
        }
    }
}

impl Indexer<Idx> for UniqueUsizeIndex {
    fn index(&self, f: Filter<Idx>) -> &[Idx] {
        if f.op() != ops::EQ {
            return &[];
        }

        match &self.0.get(*f.key()) {
            Some(Some(i)) => i,
            _ => &[],
        }
    }
}

// -----------------------------
#[derive(Debug, Default)]
pub struct MultiUsizeIndex(Vec<Option<Vec<Idx>>>);

impl KeyIdxStore<Idx> for MultiUsizeIndex {
    fn insert(&mut self, key: Idx, i: Idx) -> Result {
        if self.0.len() <= key {
            self.0.resize(key + 1, None);
        }

        match self.0[key].as_mut() {
            Some(v) => v.push(i),
            None => self.0[key] = Some(vec![i]),
        }

        Ok(())
    }
}

impl Indexer<Idx> for MultiUsizeIndex {
    fn index(&self, f: Filter<Idx>) -> &[Idx] {
        if f.op() != ops::EQ {
            return &[];
        }

        match &self.0.get(*f.key()) {
            Some(Some(v)) => v,
            _ => &[],
        }
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
            let idx = UniqueUsizeIndex::default();
            assert_eq!(0, idx.index(eq(2)).len());
            assert!(idx.0.is_empty());
        }

        #[test]
        fn find_idx_2() {
            let mut idx = UniqueUsizeIndex::default();
            idx.insert(2, 4).unwrap();

            assert_eq!(idx.index(eq(2)), &[4]);
            assert_eq!(3, idx.0.len());
        }

        #[test]
        fn or_find_idx_3_4() {
            let mut idx = UniqueUsizeIndex::default();
            idx.insert(2, 4).unwrap();
            idx.insert(4, 8).unwrap();
            idx.insert(3, 6).unwrap();

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
            let mut idx = UniqueUsizeIndex::default();
            idx.insert(2, 2).unwrap();

            assert_eq!(
                Err(IndexError::NotUniqueKey(2usize.into())),
                idx.insert(2, 2)
            );
        }

        #[test]
        fn out_of_bound() {
            let idx = UniqueUsizeIndex::default();
            assert_eq!(0, idx.filter(eq(2)).len());
        }
    }

    mod multi {
        use super::*;

        #[test]
        fn empty() {
            let idx = MultiUsizeIndex::default();
            assert_eq!(0, idx.index(eq(2)).len());
            assert!(idx.0.is_empty());
        }

        #[test]
        fn find_idx_2() {
            let mut idx = MultiUsizeIndex::default();
            idx.insert(2, 2).unwrap();

            assert!(idx.index(eq(2)).eq(&[2]));
            assert_eq!(3, idx.0.len());
        }

        #[test]
        fn double_index() {
            let mut idx = MultiUsizeIndex::default();
            idx.insert(2, 2).unwrap();
            idx.insert(2, 1).unwrap();

            assert!(idx.filter(eq(2)).eq(&[2, 1]));
        }
    }
}
