use std::ops::Deref;
use std::{borrow::Borrow, marker::PhantomData};

use crate::index::indices::{KeyIndex, MultiKeyIndex};

pub trait Filterable {
    type Key;
    type Index;

    fn contains_key(&self, key: Self::Key) -> bool;
    fn get_indices(&self, key: Self::Key) -> &[Self::Index];
}

pub trait Store {
    type Key;
    type Index;
    type Filter: Filterable<Index = Self::Index>;

    fn insert(&mut self, key: Self::Key, idx: Self::Index);
    fn delete(&mut self, key: Self::Key, idx: &Self::Index);

    fn filter(&self) -> &Self::Filter;
}

// -------------------------
pub trait ViewCreator<'a> {
    type Key;
    type Filter: Filterable;

    fn create_view<It>(&'a self, keys: It) -> View<Self::Filter>
    where
        It: IntoIterator<Item = Self::Key>;
}

/// A wrapper for a `Filterable` implementation
#[repr(transparent)]
pub struct View<F: Filterable>(F);

impl<F: Filterable> Deref for View<F> {
    type Target = F;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// -------------------------
pub struct IVec<I, K = usize, X = usize> {
    vec: Vec<Option<I>>,
    _index: PhantomData<X>,
    _key: PhantomData<K>,
}

impl<I, K, X> IVec<I, K, X> {
    pub fn new(vec: Vec<Option<I>>) -> Self {
        Self {
            vec,
            _index: PhantomData,
            _key: PhantomData,
        }
    }
}

impl<I, K, X> Filterable for IVec<I, K, X>
where
    K: Into<usize>,
    I: KeyIndex<X>,
{
    type Key = K;
    type Index = X;

    fn contains_key(&self, key: K) -> bool {
        matches!(self.vec.get((key).into()), Some(Some(_)))
    }

    fn get_indices(&self, key: K) -> &[Self::Index] {
        match self.vec.get((key).into()) {
            Some(Some(idx)) => idx.as_slice(),
            _ => &[],
        }
    }
}

impl<'a, I, K, X> Filterable for Vec<Option<(&'a I, PhantomData<K>, PhantomData<X>)>>
where
    K: Into<usize>,
    I: KeyIndex<X>,
{
    type Key = K;
    type Index = X;

    fn contains_key(&self, key: K) -> bool {
        matches!(self.get((key).into()), Some(Some(_)))
    }

    fn get_indices(&self, key: K) -> &[Self::Index] {
        match self.get((key).into()) {
            Some(Some(idx)) => idx.0.as_slice(),
            _ => &[],
        }
    }
}

impl<'a, I, K, X> ViewCreator<'a> for IVec<I, K, X>
where
    K: Into<usize>,
    I: KeyIndex<X> + 'a,
{
    type Key = K;
    type Filter = Vec<Option<(&'a I, PhantomData<K>, PhantomData<X>)>>;

    fn create_view<It>(&'a self, keys: It) -> View<Self::Filter>
    where
        It: IntoIterator<Item = Self::Key>,
    {
        let mut view = Self::Filter::new();
        view.resize(self.vec.len(), None);

        for key in keys {
            let idx: usize = key.into();
            if let Some(opt) = self.vec.get(idx) {
                view[idx] = Some((opt.as_ref().unwrap(), PhantomData, PhantomData));
            }
        }

        View(view)
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
    type Filter = Self;

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

    fn filter(&self) -> &Self::Filter {
        self
    }
}

// --------------------
use std::collections::HashMap;
use std::hash::Hash;

pub struct IMap<'a, Q: ?Sized, K = String, X = usize>(
    HashMap<K, MultiKeyIndex<X>>,
    PhantomData<&'a Q>,
);

impl<'a, Q: ?Sized, K, X> IMap<'a, Q, K, X> {
    pub fn new() -> Self {
        Self(HashMap::new(), PhantomData)
    }
}

impl<'a, Q, K, X> Filterable for IMap<'a, Q, K, X>
where
    Q: Hash + Eq + ?Sized,
    K: Borrow<Q> + Hash + Eq,
    X: Ord,
{
    type Key = &'a Q;
    type Index = X;

    fn contains_key(&self, key: Self::Key) -> bool {
        self.0.contains_key(key)
    }

    fn get_indices(&self, key: Self::Key) -> &[Self::Index] {
        match self.0.get(key) {
            Some(i) => i.as_slice(),
            None => &[],
        }
    }
}

impl<'a, Q, K, X> Filterable for (HashMap<K, &'a MultiKeyIndex<X>>, PhantomData<&'a Q>)
where
    Q: Hash + Eq + ?Sized,
    K: Borrow<Q> + Hash + Eq,
    X: Ord,
{
    type Key = &'a Q;
    type Index = X;

    fn contains_key(&self, key: Self::Key) -> bool {
        self.0.contains_key(key)
    }

    fn get_indices(&self, key: Self::Key) -> &[Self::Index] {
        match self.0.get(key) {
            Some(i) => (*i).as_slice(),
            None => &[],
        }
    }
}

impl<'a, Q, K, X> ViewCreator<'a> for IMap<'a, Q, K, X>
where
    Q: Hash + Eq + ?Sized,
    K: Borrow<Q> + Hash + Eq,
    X: Ord + 'a,
{
    type Key = K;
    type Filter = (HashMap<K, &'a MultiKeyIndex<X>>, PhantomData<&'a Q>);

    fn create_view<It>(&'a self, keys: It) -> View<Self::Filter>
    where
        It: IntoIterator<Item = Self::Key>,
    {
        let mut view = (HashMap::<K, &MultiKeyIndex<X>>::new(), PhantomData);

        for key in keys {
            if let Some(idx) = self.0.get(key.borrow()) {
                view.0.insert(key, idx);
            }
        }

        View(view)
    }
}

impl<'a, Q, K, X> Store for IMap<'a, Q, K, X>
where
    Q: Hash + Eq + ?Sized,
    K: Borrow<Q> + Hash + Eq,
    X: Ord,
{
    type Key = K;
    type Index = X;
    type Filter = Self;

    fn insert(&mut self, key: Self::Key, idx: Self::Index) {
        match self.0.get_mut(key.borrow()) {
            Some(v) => v.add(idx),
            None => {
                self.0.insert(key, MultiKeyIndex::new(idx));
            }
        }
    }

    fn delete(&mut self, key: Self::Key, idx: &Self::Index) {
        if let Some(rm_idx) = self.0.get_mut(key.borrow()) {
            if rm_idx.remove(idx) {
                self.0.remove(key.borrow());
            }
        }
    }

    fn filter(&self) -> &Self::Filter {
        self
    }
}

// --------------------
struct XList<S: Store>(S);

impl<S> XList<S>
where
    S: Store,
{
    fn contains(&self, key: <S::Filter as Filterable>::Key) -> bool {
        self.0.filter().contains_key(key)
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
        assert!(v.filter().contains_key(2));

        assert_eq!(v.get_indices(2), &['A']);
        assert_eq!(v.filter().get_indices(2), &['A']);

        let view = v.create_view([2, 100]);
        assert!(!view.contains_key(100));
        assert!(view.contains_key(2));

        assert_eq!(None, view.get_indices(100).iter().next());
        assert_eq!(&['A'], view.get_indices(2));
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

        assert_eq!(v.get_indices(1), &[String::from("A"), String::from("C")]);

        let view = v.create_view([1, 100]);
        assert!(!view.contains_key(100));
        assert!(view.contains_key(1));

        assert_eq!(None, view.get_indices(100).iter().next());
        assert_eq!(&[String::from("A"), String::from("C")], view.get_indices(1));
    }

    fn imap() {
        let mut m = IMap::new();
        m.insert(String::from("A"), 1);
        m.insert(String::from("A"), 2);

        assert!(!m.contains_key("Z"));
        assert!(m.contains_key("A"));
        assert!(m.filter().contains_key("A"));
        assert!(m.filter().contains_key(&String::from("A")));

        assert_eq!(m.get_indices("A"), &[1, 2]);
        assert_eq!(m.filter().get_indices("A"), &[1, 2]);

        let view = m.create_view([String::from("A"), String::from("ZZ")]);
        assert!(view.contains_key("A"));
        assert!(view.contains_key("A"));
        assert!(!view.contains_key(&String::from("ZZ")));

        assert_eq!(None, view.get_indices(&String::from("ZZ")).iter().next());
        assert_eq!(&[1, 2], view.get_indices(&String::from("A")));
    }

    #[test]
    fn imap_i32_char() {
        let mut m = IMap::new();
        m.insert(1, 'A');
        m.insert(2, 'B');

        assert!(!m.contains_key(&9));
        assert!(m.contains_key(&1));
        assert!(m.filter().contains_key(&2));

        assert_eq!(m.get_indices(&1), &['A']);
        assert_eq!(m.filter().get_indices(&2), &['B']);

        let view = m.create_view([1]);
        assert!(view.contains_key(&1));
        assert!(!view.contains_key(&2));

        assert_eq!(None, view.get_indices(&2).iter().next());
        assert_eq!(&['A'], view.get_indices(&1));
    }

    #[test]
    fn xlist_imap() {
        let mut m: IMap<'_, str, _, _> = IMap::new();
        m.insert(String::from("A"), 1);
        m.insert(String::from("A"), 2);
        let l = XList(m);

        assert!(!l.contains("Z"));
        assert!(l.contains("A"));
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

        assert!(!l.contains(1));
        assert!(l.contains(2));
    }
}
