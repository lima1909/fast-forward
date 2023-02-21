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
use std::fmt::Debug;

use crate::{Idx, Op};

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

/// Filter are the input data for describung a filter. A filter consist of a key and a operation [`Op`].
/// Key `K` is a unique value under which all occurring indices are stored.
///
/// For example:
/// Filter `= 5`
/// means: Op: `=` and Key: `5`
pub struct Filter<K> {
    pub op: Op,
    pub key: K,
}

impl<K> Filter<K> {
    pub fn new(op: Op, key: K) -> Self {
        Self { op, key }
    }
}

/// Find all [`Idx`] for an given [`crate::Op`] and `Key`.
pub trait IdxFilter<K> {
    fn idx(&self, f: Filter<K>) -> &[Idx];
}

/// A Store for a mapping from a given Key to one or many Indices.
pub trait KeyIdxStore<K>: IdxFilter<K> {
    fn insert(&mut self, k: K, i: Idx) -> Result;
}

type FieldValueFn<T, K> = fn(&T) -> K;

pub struct NamedStore<T, K> {
    name: &'static str,
    store: Box<dyn KeyIdxStore<K>>,
    get_field_value: FieldValueFn<T, K>,
}

impl<T, K> IdxFilter<K> for NamedStore<T, K> {
    fn idx(&self, f: Filter<K>) -> &[Idx] {
        self.store.idx(f)
    }
}

impl<T, K> IdxFilter<K> for &NamedStore<T, K> {
    fn idx(&self, f: Filter<K>) -> &[Idx] {
        self.store.idx(f)
    }
}

impl<T, K> NamedStore<T, K> {
    pub fn new(
        name: &'static str,
        store: Box<dyn KeyIdxStore<K>>,
        get_field_value: FieldValueFn<T, K>,
    ) -> Self {
        Self {
            name,
            store,
            get_field_value,
        }
    }
}

/// Collection of indices ([`KeyIdxStore`]s).
#[derive(Default)]
pub struct Indices<T>(Vec<NamedStore<T, usize>>);

impl<T> Indices<T> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add_idx(
        &mut self,
        name: &'static str,
        store: Box<dyn KeyIdxStore<usize>>,
        get_field_value: FieldValueFn<T, usize>,
    ) {
        self.0.push(NamedStore::new(name, store, get_field_value));
    }

    pub fn get_idx(&self, idx_name: &str) -> &NamedStore<T, usize> {
        self.0.iter().find(|i| i.name == idx_name).unwrap()
    }

    pub fn insert(&mut self, t: &T, idx: Idx) -> Result {
        for s in &mut self.0 {
            let key = (s.get_field_value)(t);
            s.store.insert(key, idx)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::{
        index::uint::UIntVecIndex,
        ops::eq,
        query::{IdxFilterQuery, Query},
    };

    struct Person(usize, usize, &'static str);

    #[test]
    fn person_indices() {
        let mut indices = Indices::new();
        indices.add_idx(
            "pk",
            Box::<UIntVecIndex<Unique>>::default(),
            |p: &Person| p.0,
        );
        indices.add_idx(
            "second",
            Box::<UIntVecIndex<Multi>>::default(),
            |p: &Person| p.1,
        );

        indices.insert(&Person(3, 7, "Jasmin"), 0).unwrap();
        indices.insert(&Person(41, 7, "Mario"), 1).unwrap();

        let pk = indices.get_idx("pk");

        let mut q = IdxFilterQuery::new(pk, HashSet::default());
        assert_eq!(1, q.filter(eq("", 41)).exec()[0]);
        assert_eq!(0, q.reset().filter(eq("", 3)).exec()[0]);
        assert_eq!(Vec::<usize>::new(), q.reset().filter(eq("", 101)).exec());

        let second = indices.get_idx("second");

        let mut q = IdxFilterQuery::new(second, HashSet::default());
        let r = q.filter(eq("", 7)).exec();
        assert!(r.contains(&0));
        assert!(r.contains(&1));

        let r = q.reset().filter(eq("", 3)).or(eq("", 7)).exec();
        assert!(r.contains(&0));
        assert!(r.contains(&1));
    }
}
