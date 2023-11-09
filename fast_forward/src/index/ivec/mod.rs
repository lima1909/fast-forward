#![allow(dead_code)]

use std::{
    fmt::Debug,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use self::options::{KeyIndexOptionRead, KeyIndexOptionWrite};

use super::{indices::KeyIndex, store::Filterable};

pub mod int;
mod new_filter;
mod options;
pub mod uint;

#[derive(Debug)]
#[repr(transparent)]
pub struct IVec<I, X, Opt> {
    vec: Vec<Opt>,
    _x: PhantomData<X>,
    _key_index: PhantomData<I>,
}

impl<I, X, Opt> IVec<I, X, Opt>
where
    I: KeyIndex<X>,
{
    pub(crate) const fn new() -> Self {
        Self {
            vec: Vec::new(),
            _x: PhantomData,
            _key_index: PhantomData,
        }
    }

    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            vec: Vec::with_capacity(capacity),
            _x: PhantomData,
            _key_index: PhantomData,
        }
    }

    #[inline]
    pub(crate) fn contains_key<K: Into<Key>>(&self, key: K) -> bool
    where
        Opt: KeyIndexOptionRead<I, X>,
    {
        let key = key.into();
        self.vec
            .get(key.value)
            .map_or(false, |o| o.contains(key.is_negative))
    }

    #[inline]
    pub(crate) fn get_indeces_by_key<K: Into<Key>>(&self, key: K) -> &[X]
    where
        Opt: KeyIndexOptionRead<I, X>,
    {
        let key = key.into();
        self.vec
            .get(key.value)
            .map_or(&[], |o| o.get(key.is_negative))
    }

    #[inline]
    pub(crate) fn insert<K: Into<Key>>(&mut self, key: K, index: X)
    where
        Opt: KeyIndexOptionWrite<I, X>,
    {
        let key = key.into();
        if self.vec.len() <= key.value {
            let l = if key.value == 0 { 2 } else { key.value * 2 };
            self.vec.resize(l, Opt::default());
        }
        self.vec[key.value].set(key.is_negative, index)
    }

    #[inline]
    pub(crate) fn delete<K: Into<Key>>(&mut self, key: K, index: &X)
    where
        Opt: KeyIndexOptionWrite<I, X>,
    {
        let key = key.into();
        if let Some(rm_idx) = self.vec.get_mut(key.value) {
            rm_idx.delete(key.is_negative, index)
        }
    }

    fn create_view<It>(&self, keys: It) -> IVec<I, X, Option<&I>>
    where
        It: IntoIterator<Item = Key>,
        Opt: KeyIndexOptionRead<I, X>,
    {
        let mut view = IVec::new();
        view.vec.resize(self.vec.len(), None);

        for key in keys {
            if let Some(opt) = self.vec.get(key.value) {
                view[key.value] = opt.get_opt(key.is_negative).as_ref();
            }
        }

        view
    }

    pub(crate) fn min_key_index(&self) -> Option<Opt::Output>
    where
        Opt: KeyIndexOptionRead<I, X>,
    {
        self.vec
            .iter()
            .enumerate()
            .find_map(|(pos, o)| o.map_to_position(pos))
    }

    pub(crate) fn max_key_index(&self) -> Option<Opt::Output>
    where
        Opt: KeyIndexOptionRead<I, X>,
    {
        self.vec
            .iter()
            .enumerate()
            .rev()
            .find_map(|(pos, o)| o.map_to_position(pos))
    }
}

impl<X, I, Opt> Deref for IVec<X, I, Opt> {
    type Target = Vec<Opt>;

    fn deref(&self) -> &Self::Target {
        &self.vec
    }
}

impl<X, I, Opt> DerefMut for IVec<X, I, Opt> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vec
    }
}

impl<I, X, Opt> Filterable for IVec<I, X, Opt>
where
    I: KeyIndex<X>,
    Opt: KeyIndexOptionRead<I, X>,
{
    type Key = i32;
    type Index = X;

    fn contains(&self, key: &Self::Key) -> bool {
        self.contains_key(*key)
    }

    fn get(&self, key: &Self::Key) -> &[Self::Index] {
        self.get_indeces_by_key(*key)
    }
}

#[derive(Debug)]
pub struct Key {
    value: usize,
    is_negative: bool,
}

impl From<usize> for Key {
    fn from(value: usize) -> Self {
        Self {
            value,
            is_negative: false,
        }
    }
}

impl From<i32> for Key {
    fn from(value: i32) -> Self {
        let is_negative = value < 0;
        let value = value
            .abs()
            .try_into()
            .expect("key could not convert into usize");

        Self { value, is_negative }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::indices::MultiKeyIndex;

    impl<X> IVec<MultiKeyIndex<X>, X, Option<MultiKeyIndex<X>>> {
        pub(crate) const fn new_uint() -> Self {
            Self {
                vec: Vec::new(),
                _x: PhantomData,
                _key_index: PhantomData,
            }
        }
    }

    impl<X> IVec<MultiKeyIndex<X>, X, (Option<MultiKeyIndex<X>>, Option<MultiKeyIndex<X>>)> {
        pub(crate) const fn new_int() -> Self {
            Self {
                vec: Vec::new(),
                _x: PhantomData,
                _key_index: PhantomData,
            }
        }
    }

    #[test]
    fn min_key_pos_uint() {
        let mut v = IVec::new_uint();
        assert_eq!(None, v.min_key_index());
        assert_eq!(None, v.max_key_index());

        v.insert(3, 1);
        v.insert(5, 1);
        v.insert(11, 1);

        assert_eq!(Some(3), v.min_key_index());
        assert_eq!(Some(11), v.max_key_index());

        v.insert(0, 1);
        assert_eq!(Some(0), v.min_key_index());
    }

    #[test]
    fn min_key_index_int() {
        let mut v = IVec::new_int();
        assert_eq!(None, v.min_key_index());
        assert_eq!(None, v.max_key_index());

        v.insert(3, 1);
        v.insert(-2, 1);
        v.insert(5, 1);
        v.insert(11, 1);

        assert_eq!(Some((Some(2), None)), v.min_key_index());
        assert_eq!(Some((None, Some(11))), v.max_key_index());

        v.insert(-12, 1);
        assert_eq!(Some((Some(12), None)), v.max_key_index());

        v.insert(0, 1);
        assert_eq!(Some((None, Some(0))), v.min_key_index());
    }
}
