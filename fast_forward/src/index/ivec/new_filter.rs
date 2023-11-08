use std::marker::PhantomData;

use crate::index::indices::KeyIndex;

pub trait XFilterable {
    type Key;
    type Index;

    /// Checks whether the `Key` exists.
    fn contains(&self, key: Self::Key) -> bool;

    /// Get all indices for a given `Key`.
    /// If the `Key` not exist, than this method returns `empty array`.
    fn get(&self, key: Self::Key) -> &[Self::Index];
}

pub trait XStore {
    type Key;
    type Index;
    type Filter;

    fn insert(&mut self, key: Self::Key, idx: Self::Index);
    fn delete(&mut self, key: Self::Key, idx: &Self::Index);

    fn create_filter(&self) -> &Self::Filter
    where
        Self::Filter: XFilterable;
}

pub struct IVec<'a, K, X, S> {
    vec: Vec<Option<S>>,
    _index: PhantomData<X>,
    _key: PhantomData<&'a K>,
}

impl<K, X, S> IVec<'_, K, X, S>
where
    K: Into<usize>,
    S: KeyIndex<X>,
{
    pub fn new(vec: Vec<Option<S>>) -> Self {
        Self {
            vec,
            _index: PhantomData,
            _key: PhantomData,
        }
    }
}

impl<K, X, S> From<Vec<Option<S>>> for IVec<'_, K, X, S>
where
    K: Into<usize>,
    S: KeyIndex<X>,
{
    fn from(vec: Vec<Option<S>>) -> Self {
        IVec::new(vec)
    }
}

impl<'a, K, X, S> XFilterable for IVec<'a, K, X, S>
where
    K: Into<usize> + Copy,
    S: KeyIndex<X>,
{
    type Key = &'a K;
    type Index = X;

    fn contains(&self, key: Self::Key) -> bool {
        matches!(self.vec.get((*key).into()), Some(Some(_)))
    }

    fn get(&self, key: Self::Key) -> &[Self::Index] {
        match self.vec.get((*key).into()) {
            Some(Some(idx)) => idx.as_slice(),
            _ => &[],
        }
    }
}

impl<K, X, S> XStore for IVec<'_, K, X, S>
where
    K: Into<usize>,
    S: KeyIndex<X> + Clone,
    X: Ord + Clone,
{
    type Key = K;
    type Index = X;
    type Filter = Self;

    fn insert(&mut self, key: Self::Key, idx: Self::Index) {
        let k = key.into();
        if self.vec.len() <= k {
            let dbl = if self.vec.is_empty() { k + 1 } else { k * 2 };
            self.vec.resize(dbl, None);
        }

        match self.vec[k].as_mut() {
            Some(i) => i.add(idx),
            None => self.vec[k] = Some(S::new(idx)),
        }
    }

    fn delete(&mut self, key: Self::Key, idx: &Self::Index) {
        let k = key.into();
        if let Some(Some(rm_idx)) = self.vec.get_mut(k) {
            // if the Index is the last, then remove complete Index
            if rm_idx.remove(idx) {
                self.vec[k] = None
            }
        }
    }

    fn create_filter(&self) -> &Self::Filter
    where
        Self::Filter: XFilterable,
    {
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::index::indices::{MultiKeyIndex, UniqueKeyIndex};

    use super::*;

    #[test]
    fn unique_ivec() {
        let v: IVec<usize, _, UniqueKeyIndex<char>> = IVec::new(vec![
            None,
            None,
            Some(['A'].into()),
            None,
            Some(['B'].into()),
        ]);
        assert!(!v.contains(&1usize));
        assert!(v.contains(&2));

        assert_eq!(v.get(&2), &['A']);
    }

    #[test]
    fn many_ivec() {
        let mut idxs = MultiKeyIndex::new('A');
        idxs.add('C');

        let v = IVec::new(vec![
            None,
            Some(idxs),
            None,
            None,
            Some(MultiKeyIndex::new('B')),
        ]);
        assert!(!v.contains(&0));
        assert!(v.contains(&1usize));

        assert_eq!(v.get(&1), &['A', 'C']);
    }

    // struct MyString(String);

    // impl From<&MyString> for usize {
    //     fn from(s: &MyString) -> Self {
    //         s.0.len()
    //     }
    // }

    // #[test]
    // fn mystring_unique_ivec() {
    //     let v: IVec<usize, _, UniqueKeyIndex<char>>  = IVec::new(vec![None, Some([1]), None, None, Some([2])]);
    //     assert!(!v.contains(&MyString(String::from("aa"))));
    //     assert!(v.contains(&MyString(String::from("a"))));

    //     assert_eq!(v.get(&MyString(String::from("a"))), &[1]);
    // }
}
