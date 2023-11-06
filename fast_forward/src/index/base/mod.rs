#![allow(dead_code)]

use std::{marker::PhantomData, ops::Deref};

use super::indices::KeyIndex;

mod int;
mod new_filter;
mod uint;
mod view;

pub(crate) trait KIOption<X, I>: Clone + Default
where
    I: KeyIndex<X>,
{
    type Output;

    fn contains(&self, is_negativ: bool) -> bool;
    fn get(&self, is_negativ: bool) -> &[X];
    fn set(&mut self, is_negativ: bool, index: X);
    fn delete(&mut self, is_negativ: bool, index: &X);

    fn get_opt(&self, is_negativ: bool) -> &Option<I>;
    fn get_opt_mut(&mut self, is_negativ: bool) -> &mut Option<I>;

    fn map_to_position(&self, pos: usize) -> Option<Self::Output>;
}

impl<X, I> KIOption<X, I> for Option<I>
where
    I: KeyIndex<X> + Clone,
{
    type Output = usize;

    fn contains(&self, _: bool) -> bool {
        self.is_some()
    }

    fn get(&self, _: bool) -> &[X] {
        self.as_ref().map_or(&[], |i| i.as_slice())
    }

    fn set(&mut self, _: bool, index: X) {
        match self {
            Some(idx) => idx.add(index),
            None => *self = Some(I::new(index)),
        };
    }

    fn delete(&mut self, _: bool, index: &X) {
        if let Some(rm_idx) = self {
            if rm_idx.remove(index) {
                *self = None;
            }
        }
    }

    fn get_opt(&self, _: bool) -> &Option<I> {
        self
    }

    fn get_opt_mut(&mut self, _: bool) -> &mut Option<I> {
        self
    }

    fn map_to_position(&self, pos: usize) -> Option<Self::Output> {
        self.as_ref().map(|_| pos)
    }
}

impl<X, I> KIOption<X, I> for (Option<I>, Option<I>)
where
    I: KeyIndex<X> + Clone,
{
    type Output = (Option<usize>, Option<usize>);

    fn contains(&self, is_negativ: bool) -> bool {
        self.get_opt(is_negativ).is_some()
    }

    fn get(&self, is_negativ: bool) -> &[X] {
        self.get_opt(is_negativ).get(is_negativ)
    }

    fn set(&mut self, is_negativ: bool, index: X) {
        self.get_opt_mut(is_negativ).set(is_negativ, index);
    }

    fn delete(&mut self, is_negativ: bool, index: &X) {
        self.get_opt_mut(is_negativ).delete(is_negativ, index);
    }

    fn get_opt(&self, is_negativ: bool) -> &Option<I> {
        if is_negativ {
            &self.0
        } else {
            &self.1
        }
    }

    fn get_opt_mut(&mut self, is_negativ: bool) -> &mut Option<I> {
        if is_negativ {
            &mut self.0
        } else {
            &mut self.1
        }
    }

    fn map_to_position(&self, pos: usize) -> Option<Self::Output> {
        if self.0.is_none() && self.1.is_none() {
            None
        } else {
            Some((self.0.map_to_position(pos), self.1.map_to_position(pos)))
        }
    }
}

pub struct Key {
    value: usize,
    is_negative: bool,
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

impl From<usize> for Key {
    fn from(value: usize) -> Self {
        Self {
            value,
            is_negative: false,
        }
    }
}

#[derive(Debug)]
pub(crate) struct IVec<X, I, Opt> {
    vec: Vec<Opt>,
    _x: PhantomData<X>,
    _key_index: PhantomData<I>,
}

impl<X, I, Opt> IVec<X, I, Opt>
where
    I: KeyIndex<X>,
    Opt: KIOption<X, I>,
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
    pub(crate) fn contains<K: Into<Key>>(&self, key: K) -> bool {
        let key = key.into();
        self.vec
            .get(key.value)
            .map_or(false, |o| o.contains(key.is_negative))
    }

    #[inline]
    pub(crate) fn get<K: Into<Key>>(&self, key: K) -> &[X] {
        let key = key.into();
        self.vec
            .get(key.value)
            .map_or(&[], |o| o.get(key.is_negative))
    }

    #[inline]
    pub(crate) fn insert<K: Into<Key>>(&mut self, key: K, index: X) {
        let key = key.into();
        if self.vec.len() <= key.value {
            let l = if key.value == 0 { 2 } else { key.value * 2 };
            self.vec.resize(l, Opt::default());
        }
        self.vec[key.value].set(key.is_negative, index)
    }

    #[inline]
    pub(crate) fn delete<K: Into<Key>>(&mut self, key: K, index: &X) {
        let key = key.into();
        if let Some(rm_idx) = self.vec.get_mut(key.value) {
            rm_idx.delete(false, index)
        }
    }

    pub(crate) fn min_key_index(&self) -> Option<Opt::Output> {
        self.vec
            .iter()
            .enumerate()
            .find_map(|(pos, o)| o.map_to_position(pos))
    }

    pub(crate) fn max_key_index(&self) -> Option<Opt::Output> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::indices::MultiKeyIndex;

    impl<X> IVec<X, MultiKeyIndex<X>, Option<MultiKeyIndex<X>>> {
        pub(crate) const fn new_uint() -> Self {
            Self {
                vec: Vec::new(),
                _x: PhantomData,
                _key_index: PhantomData,
            }
        }
    }

    impl<X> IVec<X, MultiKeyIndex<X>, (Option<MultiKeyIndex<X>>, Option<MultiKeyIndex<X>>)> {
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
