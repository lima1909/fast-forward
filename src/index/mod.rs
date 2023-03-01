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
//! ```text
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
pub mod idx;
pub mod map;
pub mod uint;

pub use error::IndexError;
pub use idx::{Index, Multi, Positions, Unique};
use std::ops::Deref;

use crate::{
    ops::{EQ, NE},
    query::{self, IdxFilter, Key},
    Idx, Op,
};

/// Default Result for index with the Ok(T) value or en [`IndexError`].
type Result<T = ()> = std::result::Result<T, IndexError>;

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

/// A Store for a mapping from a given Key to one or many Indices.
pub trait KeyIdxStore<K> {
    /// Insert all indices for a given `Key`.
    fn insert(&mut self, k: K, i: Idx) -> Result;

    /// find for the given `Key` all indices.
    fn find(&self, f: Filter<K>) -> &[Idx];
}

/// Find all [`Idx`] for an given [`Filter`] ([`crate::Op`]) and [`crate::query::Key`].
pub trait OpsFilter<K>: KeyIdxStore<K> {
    fn eq(&self, key: K) -> &[Idx] {
        self.find(Filter::new(EQ, key))
    }

    fn ne(&self, key: K) -> &[Idx] {
        self.find(Filter::new(NE, key))
    }
}

impl<K, S: KeyIdxStore<K>> OpsFilter<K> for S {}

type FieldValueFn<T, K> = fn(&T) -> K;

pub struct FieldIdxStore<'a, T, K> {
    field_name: &'a str,
    get_field_value_fn: FieldValueFn<T, K>,
    store: Box<dyn KeyIdxStore<K> + 'a>,
}

impl<'a, T, K> FieldIdxStore<'a, T, K> {
    pub fn new(
        field_name: &'a str,
        get_field_value_fn: FieldValueFn<T, K>,
        store: Box<dyn KeyIdxStore<K> + 'a>,
    ) -> Self {
        Self {
            field_name,
            get_field_value_fn,
            store,
        }
    }
}

impl<'a, T, K> Deref for FieldIdxStore<'a, T, K> {
    type Target = Box<dyn KeyIdxStore<K> + 'a>;

    fn deref(&self) -> &Self::Target {
        &self.store
    }
}

/// Collection of indices ([`KeyIdxStore`]s).
#[derive(Default)]
pub struct Indices<'a, T> {
    k_usize: Vec<FieldIdxStore<'a, T, usize>>,
    k_str: Vec<FieldIdxStore<'a, T, &'a str>>,
}

impl<'a, T> IdxFilter<'a> for Indices<'a, T> {
    fn filter(&self, f: query::Filter<'a>) -> &[Idx] {
        match f.key {
            Key::Usize(_u) => {
                let s = self
                    .k_usize
                    .iter()
                    .find(|i| i.field_name == f.field)
                    .unwrap();
                s.find(f.into())
            }
            Key::Str(_s) => {
                let s = self.k_str.iter().find(|i| i.field_name == f.field).unwrap();
                s.find(f.into())
            }
        }
    }
}

impl<'a, T> Indices<'a, T> {
    pub fn new() -> Self {
        Self {
            k_usize: Vec::new(),
            k_str: Vec::new(),
        }
    }
    pub fn add_usize_idx(
        &mut self,
        field_name: &'a str,
        get_field_value_fn: FieldValueFn<T, usize>,
        store: Box<dyn KeyIdxStore<usize> + 'a>,
    ) {
        self.k_usize
            .push(FieldIdxStore::new(field_name, get_field_value_fn, store))
    }

    pub fn add_str_idx(
        &mut self,
        field_name: &'a str,
        get_field_value_fn: FieldValueFn<T, &'a str>,
        store: Box<dyn KeyIdxStore<&'a str> + 'a>,
    ) {
        self.k_str
            .push(FieldIdxStore::new(field_name, get_field_value_fn, store))
    }

    pub fn get_idx(&self, idx_name: &str) -> &FieldIdxStore<T, usize> {
        self.k_usize
            .iter()
            .find(|i| i.field_name == idx_name)
            .unwrap()
    }

    pub fn insert(&mut self, t: &T, idx: Idx) -> Result {
        for s in &mut self.k_usize {
            let key = (s.get_field_value_fn)(t);
            s.store.insert(key, idx)?;
        }

        Ok(())
    }

    // pub fn query_builder<B: BinOp>(self) -> QueryBuilder<B, Self> {
    // QueryBuilder::<B, _>::new(self)
    // }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::{
        index::{
            map::UniqueStrIdx,
            uint::{PkUintIdx, UIntVecIndex},
            KeyIdxStore,
        },
        ops::eq,
        query::{Filter, IdxFilter, Key, QueryBuilder},
    };

    struct Person(usize, usize, &'static str);

    #[test]
    fn person_indices() {
        let mut indices = Indices::new();
        indices.add_usize_idx(
            "pk",
            |p: &Person| p.0,
            Box::<UIntVecIndex<Unique>>::default(),
        );
        indices.add_usize_idx(
            "second",
            |p: &Person| p.1,
            Box::<UIntVecIndex<Multi>>::default(),
        );
        // indices.add_str_idx("name", |p: &Person| p.2, Box::<UniqueStrIdx>::default());

        indices.insert(&Person(3, 7, "Jasmin"), 0).unwrap();
        indices.insert(&Person(41, 7, "Mario"), 1).unwrap();

        // let b = QueryBuilder::<HashSet<Idxs>, _>::new(indices); //indices.query_builder::<HashSet<Idx>>();
        let pk = indices.get_idx("pk");
        let b = QueryBuilder::<HashSet<Idx>, _>::new(|f: Filter| pk.find(f.into()));
        assert_eq!(1, b.query(eq("pk", 41)).exec()[0]);
        assert_eq!(0, b.query(eq("pk", 3)).exec()[0]);
        assert_eq!(Vec::<usize>::new(), b.query(eq("pk", 101)).exec());

        let second = indices.get_idx("second");
        let b = QueryBuilder::<HashSet<Idx>, _>::new(|f: Filter| second.find(f.into()));

        let r = b.query(eq("second", 7)).exec();
        assert!(r.contains(&0));
        assert!(r.contains(&1));

        let r = b.query(eq("second", 3)).or(eq("second", 7)).exec();
        assert!(r.contains(&0));
        assert!(r.contains(&1));

        // let b = QueryBuilder::<HashSet<Idx>, _>::new(indices);
        // b.query(eq("pk", 41)).exec();
    }

    struct Idxs<'a>(PkUintIdx, UniqueStrIdx<'a>);
    // struct Idxs<'a>(
    //     Box<dyn KeyIdxStore<usize> + 'a>,
    //     Box<dyn KeyIdxStore<&'a str> + 'a>,
    // );

    impl<'a> IdxFilter<'a> for Idxs<'a> {
        fn filter(&'a self, f: Filter<'a>) -> &[Idx] {
            match f.key {
                Key::Usize(_u) => self.0.find(f.into()),
                Key::Str(_s) => self.1.find(f.into()),
            }
        }
    }

    #[test]
    fn different_idxs() -> Result<()> {
        let mut idx_u = PkUintIdx::default();
        idx_u.insert(1, 1)?;
        idx_u.insert(2, 2)?;
        idx_u.insert(99, 0)?;

        let p = Person(3, 7, "a");
        let mut idx_s = UniqueStrIdx::default();
        idx_s.insert(p.2, 1)?;
        idx_s.insert("b", 2)?;
        idx_s.insert("z", 0)?;

        let idxs = Idxs(idx_u, idx_s);
        // let idxs = Idxs(Box::new(idx_u), Box::new(idx_s));

        let b = QueryBuilder::<HashSet<Idx>, _>::new(idxs);
        let r = b.query(eq("", 1)).and(eq("", "a")).exec();
        assert_eq!(&[1], &r[..]);

        let r = b.query(eq("", "z")).or(eq("", 1)).and(eq("", "a")).exec();
        // = "z" or = 1 and = "a" => (= 1 and "a") or "z"
        assert!(r.contains(&1));
        assert!(r.contains(&0));

        Ok(())
    }
}
