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

use crate::{
    ops::{EQ, NE},
    query::{Key, NamedPredicate, Queryable},
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Predicate<'k> {
    pub op: Op,
    pub key: Key<'k>,
}

impl<'k> Predicate<'k> {
    pub fn new(op: Op, key: Key<'k>) -> Self {
        Self { op, key }
    }

    pub fn new_eq(key: Key<'k>) -> Self {
        Self { op: EQ, key }
    }
}

/// A Store for a mapping from a given Key to one or many Indices.
pub trait Store<'k> {
    /// Insert all indices for a given `Key`.
    fn insert(&mut self, k: Key<'k>, i: Idx) -> Result;
}

/// Filtering the [`Store`] with a given [`Predicate`]
pub trait Filterable<'k> {
    /// find for the given `Key` all indices.
    fn filter(&self, p: Predicate<'k>) -> &[Idx];
}

/// Find all [`Idx`] for an given [`Predicate`] ([`crate::Op`]) and [`crate::query::Key`].
pub trait OpsFilter<'k>: Filterable<'k> {
    fn eq<K: Into<Key<'k>>>(&self, k: K) -> &[Idx] {
        self.filter(Predicate::new_eq(k.into()))
    }

    fn ne<K: Into<Key<'k>>>(&self, k: K) -> &[Idx] {
        self.filter(Predicate::new(NE, k.into()))
    }
}

impl<'k, F: Filterable<'k>> OpsFilter<'k> for F {}

pub trait FilterableStore<'k>: Store<'k> + Filterable<'k> {}

impl<'k, F: Store<'k> + Filterable<'k>> FilterableStore<'k> for F {}

type FieldValueFn<'k, T> = fn(&T) -> Key<'k>;

/// `FieldStore` extend a [`Store`] with an field-name and a function to get the value of an given object-type `<T>`
pub struct FieldStore<'k, T> {
    field: &'static str,
    field_value_fn: FieldValueFn<'k, T>,
    pub store: Box<dyn FilterableStore<'k> + 'k>,
}

impl<'k, T> FieldStore<'k, T> {
    pub fn new(
        field: &'static str,
        field_value_fn: FieldValueFn<'k, T>,
        store: Box<dyn FilterableStore<'k> + 'k>,
    ) -> Self {
        Self {
            field,
            field_value_fn,
            store,
        }
    }
}

/// Collection of indices ([`FieldStore`]s).
#[derive(Default)]
pub struct Indices<'i, T>(Vec<FieldStore<'i, T>>);

impl<'k, T> Queryable<'k> for Indices<'k, T> {
    fn filter<P>(&self, p: P) -> &[Idx]
    where
        P: Into<NamedPredicate<'k>>,
    {
        let f: NamedPredicate = p.into();
        let s = self.0.iter().find(|s| s.field == f.field).unwrap();
        s.store.filter(f.p)
    }
}

impl<'i, T> Indices<'i, T> {
    pub fn new(
        field: &'static str,
        field_value_fn: FieldValueFn<'i, T>,
        store: Box<dyn FilterableStore<'i> + 'i>,
    ) -> Self {
        let mut s = Self(Vec::new());
        s.add_idx(field, field_value_fn, store);
        s
    }

    pub fn add_idx(
        &mut self,
        field: &'static str,
        field_value_fn: FieldValueFn<'i, T>,
        store: Box<dyn FilterableStore<'i> + 'i>,
    ) {
        self.0.push(FieldStore::new(field, field_value_fn, store))
    }

    pub fn get_idx(&self, idx_name: &str) -> &FieldStore<'i, T> {
        self.0.iter().find(|i| i.field == idx_name).unwrap()
    }

    pub fn insert(&mut self, t: &T, idx: Idx) -> Result {
        for s in &mut self.0 {
            let key = (s.field_value_fn)(t);
            s.store.insert(key, idx)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        index::{
            map::UniqueStrIdx,
            uint::{PkUintIdx, UIntVecIndex},
        },
        ops::eq,
        query::Key,
    };
    use std::collections::HashSet;

    #[derive(Default, Debug)]
    struct Person(usize, usize, &'static str);

    #[test]
    fn person_indices() {
        let mut indices = Indices::new(
            "pk",
            |p: &Person| Key::Usize(p.0),
            Box::<UIntVecIndex<Unique>>::default(),
        );
        indices.add_idx(
            "second",
            |p: &Person| Key::Usize(p.1),
            Box::<UIntVecIndex<Multi>>::default(),
        );
        indices.add_idx(
            "name",
            |p: &Person| Key::Str(p.2),
            Box::<UniqueStrIdx>::default(),
        );

        indices.insert(&Person(3, 7, "Jasmin"), 0).unwrap();
        indices.insert(&Person(41, 7, "Mario"), 1).unwrap();

        let b = indices.query_builder::<HashSet<Idx>>();

        assert_eq!(1, b.query(eq("pk", 41)).exec()[0]);
        assert_eq!(0, b.query(eq("pk", 3)).exec()[0]);
        assert_eq!(Vec::<usize>::new(), b.query(eq("pk", 101)).exec());

        let r = b.query(eq("second", 7)).exec();
        assert!(r.contains(&0));
        assert!(r.contains(&1));

        let r = b.query(eq("second", 3)).or(eq("second", 7)).exec();
        assert!(r.contains(&0));
        assert!(r.contains(&1));

        let r = b.query(eq("name", "Jasmin")).exec();
        assert_eq!(r, vec![0]);

        let r = b.query(eq("name", "Jasmin")).or(eq("name", "Mario")).exec();
        assert!(r.contains(&0));
        assert!(r.contains(&1));
    }

    struct Idxs<'k>(
        Box<dyn FilterableStore<'k> + 'k>,
        Box<dyn FilterableStore<'k> + 'k>,
    );

    impl<'k> Filterable<'k> for Idxs<'k> {
        fn filter(&self, p: Predicate<'k>) -> &[Idx] {
            match p.key {
                Key::Usize(_u) => self.0.filter(p),
                Key::Str(_s) => self.1.filter(p),
            }
        }
    }

    #[test]
    fn different_idxs() -> Result<()> {
        let mut idx_u = PkUintIdx::default();
        idx_u.insert_idx(1, 1)?;
        idx_u.insert_idx(2, 2)?;
        idx_u.insert_idx(99, 0)?;

        let p = Person(3, 7, "a");
        let mut idx_s = UniqueStrIdx::default();
        idx_s.insert_str(p.2, 1)?;
        idx_s.insert_str("b", 2)?;
        idx_s.insert_str("z", 0)?;

        let idxs = Idxs(Box::new(idx_u), Box::new(idx_s));

        let b = idxs.query_builder::<HashSet<Idx>>();
        let r = b.query(eq("", 1)).and(eq("", "a")).exec();
        assert_eq!(&[1], &r[..]);

        let r = b.query(eq("", "z")).or(eq("", 1)).and(eq("", "a")).exec();
        // = "z" or = 1 and = "a" => (= 1 and "a") or "z"
        assert!(r.contains(&1));
        assert!(r.contains(&0));

        Ok(())
    }

    #[test]
    fn collect_idxfilters() {
        let p = Person(3, 7, "a");
        let mut idx_s = UniqueStrIdx::default();
        idx_s.insert_str(p.2, 1).unwrap();

        let idxs = Idxs(Box::<PkUintIdx>::default(), Box::new(idx_s));

        let v: Vec<Box<dyn Filterable>> = vec![
            Box::<UniqueStrIdx>::default(),
            Box::<PkUintIdx>::default(),
            Box::new(idxs),
        ];
        assert_eq!(3, v.len());
    }
}
