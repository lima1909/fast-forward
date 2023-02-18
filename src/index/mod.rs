//! The purpose of an Index is to find faster a specific item in a list (Slice, Vec, ...).
//! This means, it does not have to touch and compare every item in the list.
//!
//! An Index has two parts, a `Key` (item to search for) and a position (the index in the list) [`Idx`].
//!
//! There are two types of Index:
//! - `Unique Index`: for a given `Key` exist exactly one [`Idx`].
//! - `Multi Index` : for a given `Key` exists many [`Idx`]s.
//!
//! # Example for an Vec-Multi-Index:
//!
//! Map-Index:
//!
//! - `Key` = name (String)
//! - [`Idx`] = index is the position in a List (Vec)
//!
//! ```java
//! let _names = vec!["Paul", "Jasmin", "Inge", "Paul", ...];
//!
//!  Key       | Idx
//! -------------------
//!  "Jasmin"  | 1
//!  "Paul"    | 0, 3
//!  "Inge"    | 2
//!   ...      | ...
//! ```

#![allow(dead_code)]
pub mod error;
pub mod map;
pub mod uint;

pub use error::IndexError;
use std::{fmt::Debug, marker::PhantomData, ops::Deref};

use crate::{Idx, IdxFilter};

/// Default Result for index with the Ok(T) value or en [`IndexError`].
type Result<T = ()> = std::result::Result<T, IndexError>;

pub trait Index: Debug {
    fn new(i: Idx) -> Self;
    fn add(&mut self, i: Idx) -> Result;
    fn get(&self) -> &[Idx];
}

#[derive(Debug, Default, Clone)]
pub struct Unique([Idx; 1]);

impl Index for Unique {
    #[inline]
    fn new(i: Idx) -> Self {
        Unique([i])
    }

    #[inline]
    fn add(&mut self, _i: Idx) -> Result {
        Err(IndexError::NotUniqueKey)
    }

    #[inline]
    fn get(&self) -> &[Idx] {
        &self.0
    }
}

#[derive(Debug, Default, Clone)]
pub struct Multi(Vec<Idx>);

impl Index for Multi {
    #[inline]
    fn new(i: Idx) -> Self {
        Multi(vec![i])
    }

    #[inline]
    fn add(&mut self, i: Idx) -> Result {
        self.0.push(i);
        Ok(())
    }

    #[inline]
    fn get(&self) -> &[Idx] {
        &self.0
    }
}

/// A Store for a mapping from a given Key to one or many Indices.
pub trait KeyIdxStore<K>: IdxFilter<K> {
    fn insert(&mut self, k: K, i: Idx) -> Result;
}

// fn filter(k: Key, op: crate::Op) {
//     let _vu: Vec<&dyn KeyIdxStore<usize>> = vec![];
//     let _vs: Vec<&dyn KeyIdxStore<String>> = vec![];

//     let _r = match k {
//         Key::Usize(u) => _vu.get(0).unwrap().filter(&u, op),
//         Key::String(s) => _vs.get(0).unwrap().filter(&s, op),
//         Key::I32(_) => todo!(),
//     };
// }

pub struct NamedStore<T, K, F> {
    name: &'static str,
    store: Box<dyn KeyIdxStore<K>>,
    get_field_value: F,
    _type: PhantomData<T>,
}

impl<T, K, F> NamedStore<T, K, F> {
    pub fn new(name: &'static str, store: Box<dyn KeyIdxStore<K>>, get_field_value: F) -> Self {
        Self {
            name,
            store,
            get_field_value,
            _type: PhantomData,
        }
    }
}

impl<T, K, F> Deref for NamedStore<T, K, F> {
    type Target = dyn KeyIdxStore<K>;

    fn deref(&self) -> &Self::Target {
        self.store.as_ref()
    }
}

/// Collection of indices ([`KeyIdxStore`]s).
#[derive(Default)]
pub struct Indices<T, F>(Vec<NamedStore<T, usize, F>>);

impl<T, F> Indices<T, F> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add(
        &mut self,
        name: &'static str,
        store: Box<dyn KeyIdxStore<usize>>,
        get_field_value: F,
    ) {
        self.0.push(NamedStore::new(name, store, get_field_value));
    }

    pub fn store(&self, idx_name: &str) -> &NamedStore<T, usize, F> {
        self.0.iter().find(|i| i.name == idx_name).unwrap()
    }

    pub fn insert_index(&mut self, idx_name: &str, t: &T, idx: Idx) -> Result
    where
        F: Fn(&T) -> usize,
    {
        for s in &mut self.0 {
            if s.name == idx_name {
                let key = (s.get_field_value)(t);
                s.store.insert(key, idx)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{index::uint::UIntVecIndex, ops::eq};

    struct Person(usize, &'static str);

    #[test]
    fn person_indices() {
        let mut indices = Indices::new();
        indices.add(
            "pk",
            Box::<UIntVecIndex<Unique>>::default(),
            |p: &Person| p.0,
        );

        indices.insert_index("pk", &Person(3, "Jasmin"), 0).unwrap();
        indices.insert_index("pk", &Person(41, "Mario"), 1).unwrap();

        let idx = indices.store("pk");
        assert_eq!(1, idx.idx(eq(41))[0]);
        assert_eq!(0, idx.idx(eq(3))[0]);

        assert!(idx.idx(eq(101)).eq(&[]));
    }
}
