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
pub mod idx;
pub mod map;
pub mod uint;

pub use idx::{And, Index, Multi, Positions, Unique};

use crate::{query::Queryable, Idx, Key, Op, Predicate, Result};

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

pub trait FilterableStore<'k>: Store<'k> + Filterable<'k> {}

impl<'k, F: Store<'k> + Filterable<'k>> FilterableStore<'k> for F {}

/// Find all [`Idx`] for an given [`Predicate`] ([`crate::Op`]) and [`crate::Key`].
pub trait OpsFilter<'k>: Filterable<'k> {
    fn eq<K: Into<Key<'k>>>(&self, k: K) -> &[Idx] {
        self.filter(Predicate::new_eq(k.into()))
    }

    fn ne<K: Into<Key<'k>>>(&self, k: K) -> &[Idx] {
        self.filter(Predicate::new(Op::NE, k.into()))
    }
}

impl<'k, F: Filterable<'k>> OpsFilter<'k> for F {}

type FieldValueFn<'k, T> = fn(&'k T) -> Key<'k>;

/// `FieldStore` extend a [`Store`] with an field-name and a function to get the value of an given object-type `<T>`
pub struct FieldStore<'k, T> {
    field: &'k str,
    field_value_fn: FieldValueFn<'k, T>,
    pub store: Box<dyn FilterableStore<'k> + 'k>,
}

impl<'k, T> FieldStore<'k, T> {
    pub const fn new(
        field: &'k str,
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
        P: Into<Predicate<'k>>,
    {
        let p: Predicate = p.into();
        self.get_idx(p.0).store.filter(p)
    }
}

impl<'i, T> Indices<'i, T> {
    pub fn new(
        field: &'i str,
        field_value_fn: FieldValueFn<'i, T>,
        store: impl FilterableStore<'i> + 'i,
    ) -> Self {
        let mut s = Self(Vec::new());
        s.add_idx(field, field_value_fn, store);
        s
    }

    pub fn add_idx(
        &mut self,
        field: &'i str,
        field_value_fn: FieldValueFn<'i, T>,
        store: impl FilterableStore<'i> + 'i,
    ) {
        self.0
            .push(FieldStore::new(field, field_value_fn, Box::new(store)))
    }

    pub fn get_idx(&self, idx_name: &str) -> &FieldStore<'i, T> {
        self.0.iter().find(|i| i.field == idx_name).unwrap()
    }

    pub fn insert(&mut self, t: &'i T, idx: Idx) -> Result {
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
        eq,
        index::{
            map::UniqueStrIdx,
            uint::{PkUintIdx, UIntVecIndex},
        },
        Key,
    };
    use std::collections::HashSet;

    #[derive(Debug, Clone, Copy)]
    enum Gender {
        Male,
        Female,
        None,
    }

    impl<'a> From<Gender> for Key<'a> {
        fn from(g: Gender) -> Self {
            match g {
                Gender::None => Key::Usize(0),
                Gender::Male => Key::Usize(1),
                Gender::Female => Key::Usize(2),
            }
        }
    }

    #[derive(Debug)]
    struct Person(usize, usize, String, Gender);

    #[test]
    fn person_indices() {
        let persons = vec![
            Person(3, 7, "Jasmin".to_string(), Gender::Female),
            Person(41, 7, "Mario".to_string(), Gender::Male),
            Person(111, 234, "Paul".to_string(), Gender::Male),
        ];

        let mut indices = Indices::new(
            "pk",
            |p: &Person| p.0.into(),
            UIntVecIndex::<Unique>::default(),
        );
        indices.add_idx(
            "second",
            |p: &Person| p.1.into(),
            UIntVecIndex::<Multi>::default(),
        );
        indices.add_idx("name", |p: &Person| (&p.2).into(), UniqueStrIdx::default());
        indices.add_idx(
            "gender",
            |p: &Person| p.3.into(),
            UIntVecIndex::<Multi>::default(),
        );

        indices.insert(&persons[0], 0).unwrap();
        indices.insert(&persons[1], 1).unwrap();
        indices.insert(&persons[2], 2).unwrap();

        let b = indices.query_builder::<HashSet<Idx>>();

        assert_eq!(Some(1), b.query(eq("pk", 41)).exec().next());
        assert_eq!(Some(0), b.query(eq("pk", 3)).exec().next());
        assert_eq!(None, b.query(eq("pk", 101)).exec().next());

        let r: Vec<Idx> = b.query(eq("second", 7)).exec().collect();
        assert!(r.contains(&0));
        assert!(r.contains(&1));

        let r: Vec<Idx> = b
            .query(eq("second", 3))
            .or(eq("second", 7))
            .exec()
            .collect();
        assert!(r.contains(&0));
        assert!(r.contains(&1));

        let r = b.query(eq("name", "Jasmin")).exec().next();
        assert_eq!(Some(0), r);

        let r: Vec<Idx> = b
            .query(eq("name", "Jasmin"))
            .or(eq("name", "Mario"))
            .exec()
            .collect();
        assert!(r.contains(&0));
        assert!(r.contains(&1));

        let r: Vec<Idx> = b.query(eq("gender", Gender::Male)).exec().collect();
        assert!(r.contains(&2));
        assert!(r.contains(&1));
        let r = b.query(eq("gender", Gender::Female)).exec().next();
        assert_eq!(r, Some(0));
    }

    struct Idxs<'k>(
        Box<dyn FilterableStore<'k> + 'k>,
        Box<dyn FilterableStore<'k> + 'k>,
    );

    impl<'k> Filterable<'k> for Idxs<'k> {
        fn filter(&self, p: Predicate<'k>) -> &[Idx] {
            match &p.2 {
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

        let p = Person(3, 7, "a".to_string(), Gender::None);
        let mut idx_s = UniqueStrIdx::default();
        idx_s.insert_str(&p.2, 1)?;
        idx_s.insert_str("b", 2)?;
        idx_s.insert_str("z", 0)?;

        let idxs = Idxs(Box::new(idx_u), Box::new(idx_s));

        let b = idxs.query_builder::<HashSet<Idx>>();
        let mut r = b.query(1).and("a").exec();
        assert_eq!(1, r.next().unwrap());

        let r: Vec<Idx> = b.query("z").or(1).and("a").exec().collect();
        // = "z" or = 1 and = "a" => (= 1 and "a") or "z"
        assert!(r.contains(&1));
        assert!(r.contains(&0));

        Ok(())
    }

    #[test]
    fn collect_idxfilters() {
        let p = Person(3, 7, "a".to_string(), Gender::None);
        let mut idx_s = UniqueStrIdx::default();
        idx_s.insert_str(&p.2, 1).unwrap();

        let idxs = Idxs(Box::<PkUintIdx>::default(), Box::new(idx_s));

        let v: Vec<Box<dyn Filterable>> = vec![
            Box::<UniqueStrIdx>::default(),
            Box::<PkUintIdx>::default(),
            Box::new(idxs),
        ];
        assert_eq!(3, v.len());
    }
}
