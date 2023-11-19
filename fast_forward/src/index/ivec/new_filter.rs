use std::{borrow::Borrow, marker::PhantomData};

use crate::index::indices::{KeyIndex, MultiKeyIndex};

pub trait XFilterable<Key> {
    type Index;

    fn contains(&self, key: Key) -> bool;
    fn get(&self, key: Key) -> &[Self::Index];
}

pub trait XStore {
    type Key;
    type Index;

    fn insert(&mut self, key: Self::Key, idx: Self::Index);
    fn delete(&mut self, key: Self::Key, idx: &Self::Index);
}

pub struct IVec<I, K = usize, X = usize> {
    vec: Vec<Option<I>>,
    _index: PhantomData<X>,
    _key: PhantomData<K>,
}

impl<I, K, X> IVec<I, K, X>
where
    K: Into<usize>,
    I: KeyIndex<X>,
{
    pub fn new(vec: Vec<Option<I>>) -> Self {
        Self {
            vec,
            _index: PhantomData,
            _key: PhantomData,
        }
    }
}

impl<I, K, X> XFilterable<K> for IVec<I, K, X>
where
    K: Into<usize>,
    I: KeyIndex<X>,
{
    type Index = X;

    fn contains<'a>(&self, key: K) -> bool {
        matches!(self.vec.get((key).into()), Some(Some(_)))
    }

    fn get<'a>(&self, key: K) -> &[Self::Index] {
        match self.vec.get((key).into()) {
            Some(Some(idx)) => idx.as_slice(),
            _ => &[],
        }
    }
}

impl<I, K, X> XStore for IVec<I, K, X>
where
    K: Into<usize>,
    I: KeyIndex<X> + Clone,
    X: Ord + Clone,
{
    type Key = K;
    type Index = X;

    fn insert(&mut self, key: Self::Key, idx: Self::Index) {
        let k = key.into();
        if self.vec.len() <= k {
            let dbl = if self.vec.is_empty() { k + 1 } else { k * 2 };
            self.vec.resize(dbl, None);
        }

        match self.vec[k].as_mut() {
            Some(i) => i.add(idx),
            None => self.vec[k] = Some(I::new(idx)),
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
}

// --------------------
use std::collections::HashMap;
use std::hash::Hash;

pub struct IMap<K = String, X = usize>(HashMap<K, MultiKeyIndex<X>>);

impl<K, X> IMap<K, X> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}

impl<K, X, Q> XFilterable<&Q> for IMap<K, X>
where
    Q: Hash + Eq + ?Sized,
    K: Borrow<Q> + Hash + Eq,
    X: Ord + PartialEq,
{
    type Index = X;

    fn contains(&self, key: &Q) -> bool {
        self.0.contains_key(key)
    }

    fn get(&self, key: &Q) -> &[Self::Index] {
        match self.0.get(key) {
            Some(i) => i.as_slice(),
            None => &[],
        }
    }
}

impl<K, X> XStore for IMap<K, X>
where
    K: Hash + Eq,
    X: Ord,
{
    type Key = K;
    type Index = X;

    fn insert(&mut self, key: Self::Key, idx: Self::Index) {
        match self.0.get_mut(&key) {
            Some(v) => v.add(idx),
            None => {
                self.0.insert(key, MultiKeyIndex::new(idx));
            }
        }
    }

    fn delete(&mut self, key: Self::Key, idx: &Self::Index) {
        if let Some(rm_idx) = self.0.get_mut(&key) {
            if rm_idx.remove(idx) {
                self.0.remove(&key);
            }
        }
    }
}

// --------------------
struct XList<S: XStore>(S);

impl<S> XList<S>
where
    S: XStore,
{
    fn contains<K>(&self, key: K) -> bool
    where
        S: XFilterable<K>,
    {
        self.0.contains(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::indices::{MultiKeyIndex, UniqueKeyIndex};

    #[test]
    fn unique_ivec() {
        let v: IVec<UniqueKeyIndex<char>, u8, _> = IVec::new(vec![
            None,
            None,
            Some(['A'].into()),
            None,
            Some(['B'].into()),
        ]);
        assert!(!v.contains(1));
        assert!(v.contains(2));

        assert_eq!(v.get(2), &['A']);
    }

    #[test]
    fn many_ivec() {
        let mut idx = MultiKeyIndex::new(String::from("A"));
        idx.add(String::from("C"));

        let v: IVec<MultiKeyIndex<String>, usize, _> = IVec::new(vec![
            None,
            Some(idx),
            None,
            None,
            Some(MultiKeyIndex::new("B".into())),
        ]);
        assert!(!v.contains(0));
        assert!(v.contains(1));

        assert_eq!(v.get(1), &[String::from("A"), String::from("C")]);
    }

    #[test]
    fn imap() {
        let mut m: IMap<String, usize> = IMap::new();
        m.insert(String::from("A"), 1);
        m.insert(String::from("A"), 2);

        assert!(m.contains("A"));
        assert!(!m.contains("Z"));

        assert_eq!(m.get("A"), &[1, 2]);
    }

    #[test]
    fn xlist_imap() {
        let mut m: IMap<String, usize> = IMap::new();
        m.insert(String::from("A"), 1);
        m.insert(String::from("A"), 2);
        let l = XList(m);

        assert!(l.contains("A"));
        assert!(!l.contains("Z"));
    }

    #[test]
    fn xlist_ivec() {
        let v: IVec<UniqueKeyIndex<char>, u8, _> = IVec::new(vec![
            None,
            None,
            Some(['A'].into()),
            None,
            Some(['B'].into()),
        ]);
        let l = XList(v);

        assert!(l.contains(2));
        assert!(!l.contains(1));
    }
}
