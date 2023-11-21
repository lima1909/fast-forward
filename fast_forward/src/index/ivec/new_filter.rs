use std::{borrow::Borrow, marker::PhantomData};

use crate::index::indices::{KeyIndex, MultiKeyIndex};

pub trait Filterable<Key> {
    type Index;

    fn contains_key(&self, key: Key) -> bool;
    fn get_indices_by_key(&self, key: Key) -> &[Self::Index];
}

pub trait Store {
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

impl<I, K, X> Filterable<K> for IVec<I, K, X>
where
    K: Into<usize>,
    I: KeyIndex<X>,
{
    type Index = X;

    fn contains_key<'a>(&self, key: K) -> bool {
        matches!(self.vec.get((key).into()), Some(Some(_)))
    }

    fn get_indices_by_key<'a>(&self, key: K) -> &[Self::Index] {
        match self.vec.get((key).into()) {
            Some(Some(idx)) => idx.as_slice(),
            _ => &[],
        }
    }
}

impl<'a, I, K, X> Filterable<K> for Vec<Option<(&'a I, PhantomData<X>)>>
where
    K: Into<usize>,
    I: KeyIndex<X>,
{
    type Index = X;

    fn contains_key(&self, key: K) -> bool {
        matches!(self.get((key).into()), Some(Some(_)))
    }

    fn get_indices_by_key(&self, key: K) -> &[Self::Index] {
        match self.get((key).into()) {
            Some(Some(idx)) => idx.0.as_slice(),
            _ => &[],
        }
    }
}

impl<'a, I, K, X> ViewCreator<'a, K> for IVec<I, K, X>
where
    K: Into<usize>,
    I: KeyIndex<X> + 'a,
{
    type Filter = Vec<Option<(&'a I, PhantomData<X>)>>;

    fn create_view<It>(&'a self, keys: It) -> View<K, Self::Filter>
    where
        It: IntoIterator<Item = K>,
    {
        let mut view = Self::Filter::new();
        view.resize(self.vec.len(), None);

        for key in keys {
            let idx: usize = key.into();
            if let Some(opt) = self.vec.get(idx) {
                view[idx] = Some((opt.as_ref().unwrap(), PhantomData));
            }
        }

        View::new(view)
    }
}

impl<I, K, X> Store for IVec<I, K, X>
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

impl<K, X, Q> Filterable<&Q> for IMap<K, X>
where
    Q: Hash + Eq + ?Sized,
    K: Borrow<Q> + Hash + Eq,
    X: Ord + PartialEq,
{
    type Index = X;

    fn contains_key(&self, key: &Q) -> bool {
        self.0.contains_key(key)
    }

    fn get_indices_by_key(&self, key: &Q) -> &[Self::Index] {
        match self.0.get(key) {
            Some(i) => i.as_slice(),
            None => &[],
        }
    }
}

impl<'a, K, X> Filterable<K> for HashMap<K, &'a MultiKeyIndex<X>>
where
    K: Hash + Eq,
    X: Ord + PartialEq,
{
    type Index = X;

    fn contains_key(&self, key: K) -> bool {
        self.contains_key(&key)
    }

    fn get_indices_by_key(&self, key: K) -> &[Self::Index] {
        match self.get(&key) {
            Some(i) => i.as_slice(),
            None => &[],
        }
    }
}

impl<'a, K, X> ViewCreator<'a, K> for IMap<K, X>
where
    K: Hash + Eq,
    X: Ord + PartialEq + 'a,
{
    type Filter = HashMap<K, &'a MultiKeyIndex<X>>;

    fn create_view<It>(&'a self, keys: It) -> View<K, Self::Filter>
    where
        It: IntoIterator<Item = K>,
    {
        let mut view = Self::Filter::new();

        for key in keys {
            if let Some(idx) = self.0.get(&key) {
                view.insert(key, idx);
            }
        }

        View::new(view)
    }
}

impl<K, X> Store for IMap<K, X>
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
struct XList<S: Store>(S);

impl<S> XList<S>
where
    S: Store,
{
    fn contains<K>(&self, key: K) -> bool
    where
        S: Filterable<K>,
    {
        self.0.contains_key(key)
    }
}

// -------------------------
pub trait ViewCreator<'a, K> {
    type Filter: Filterable<K>;

    fn create_view<It>(&'a self, keys: It) -> View<K, Self::Filter>
    where
        It: IntoIterator<Item = K>;
}

/// A wrapper for a `Filterable` implementation
#[repr(transparent)]
pub struct View<K, F: Filterable<K>>(pub(crate) F, PhantomData<K>);

impl<K, F: Filterable<K>> View<K, F> {
    pub fn new(filter: F) -> Self {
        Self(filter, PhantomData)
    }
}

impl<K, F: Filterable<K>> Filterable<K> for View<K, F> {
    type Index = F::Index;

    fn contains_key(&self, key: K) -> bool {
        self.0.contains_key(key)
    }

    fn get_indices_by_key(&self, key: K) -> &[Self::Index] {
        self.0.get_indices_by_key(key)
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
        assert!(!v.contains_key(1));
        assert!(v.contains_key(2));

        assert_eq!(v.get_indices_by_key(2), &['A']);

        let view = v.create_view([2, 100]);
        assert!(!view.contains_key(100));
        assert!(view.contains_key(2));

        assert_eq!(None, view.get_indices_by_key(100).iter().next());
        assert_eq!(&['A'], view.get_indices_by_key(2));
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
        assert!(!v.contains_key(0));
        assert!(v.contains_key(1));

        assert_eq!(
            v.get_indices_by_key(1),
            &[String::from("A"), String::from("C")]
        );

        let view = v.create_view([1, 100]);
        assert!(!view.contains_key(100));
        assert!(view.contains_key(1));

        assert_eq!(None, view.get_indices_by_key(100).iter().next());
        assert_eq!(
            &[String::from("A"), String::from("C")],
            view.get_indices_by_key(1)
        );
    }

    #[test]
    fn imap() {
        let mut m: IMap<String, usize> = IMap::new();
        m.insert(String::from("A"), 1);
        m.insert(String::from("A"), 2);

        assert!(m.contains_key("A"));
        assert!(!m.contains_key("Z"));

        assert_eq!(m.get_indices_by_key("A"), &[1, 2]);

        let view = m.create_view([String::from("A"), String::from("ZZ")]);
        assert!(view.contains_key(String::from("A")));
        assert!(!view.contains_key(String::from("ZZ")));

        assert_eq!(
            None,
            view.get_indices_by_key(String::from("ZZ")).iter().next()
        );
        assert_eq!(&[1, 2], view.get_indices_by_key(String::from("A")));
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
