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

use std::borrow::Cow;

pub use idx::{Index, Multi, Unique};

use crate::{query::Queryable, Idx, Key, Op, Predicate, Result};

/// A Store for a mapping from a given Key to one or many Indices.
pub trait Store<'s> {
    /// Insert all indices for a given `Key`.
    fn insert(&mut self, k: Key<'s>, i: Idx) -> Result;
}

/// Filtering the [`Store`] with a given [`Predicate`]
pub trait Filterable<'k> {
    /// find for the given `Key` all indices.
    fn filter(&self, p: Predicate<'k>) -> Result<Cow<[usize]>>;
}

pub trait FilterableStore<'k, 's>: Store<'s> + Filterable<'k> {}

impl<'k, 's, F: Store<'s> + Filterable<'k>> FilterableStore<'k, 's> for F {}

/// Find all [`Idx`] for an given [`Predicate`] ([`crate::Op`]) and [`crate::Key`].
pub trait OpsFilter<'k>: Filterable<'k> {
    fn eq<K: Into<Key<'k>>>(&self, k: K) -> Result<Cow<[usize]>> {
        self.filter(Predicate::new_eq(k.into()))
    }

    fn ne<K: Into<Key<'k>>>(&self, k: K) -> Result<Cow<[usize]>> {
        self.filter(Predicate::new(Op::NE, k.into()))
    }
}

impl<'k, F: Filterable<'k>> OpsFilter<'k> for F {}

type FieldValueFn<'s, T> = fn(&'s T) -> Key<'s>;

/// `FieldStore` extend a [`Store`] with an field-name and a function to get the value of an given object-type `<T>`
pub struct FieldStore<'k, 's, T> {
    field: &'s str,
    field_value_fn: FieldValueFn<'s, T>,
    pub store: Box<dyn FilterableStore<'k, 's> + 's>,
}

impl<'k, 's, T> FieldStore<'k, 's, T> {
    pub const fn new(
        field: &'s str,
        field_value_fn: FieldValueFn<'s, T>,
        store: Box<dyn FilterableStore<'k, 's> + 's>,
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
pub struct Indices<'k, 'i, T>(Vec<FieldStore<'k, 'i, T>>);

impl<'k, 'i, T> Queryable<'k> for Indices<'k, 'i, T> {
    fn filter<P>(&self, p: P) -> Result<Cow<[usize]>>
    where
        P: Into<Predicate<'k>>,
    {
        let p: Predicate = p.into();
        self.get_idx(p.0).store.filter(p)
    }
}

impl<'k, 'i, T> Indices<'k, 'i, T> {
    pub fn new(
        field: &'i str,
        field_value_fn: FieldValueFn<'i, T>,
        store: impl FilterableStore<'k, 'i> + 'i,
    ) -> Self {
        let mut s = Self(Vec::new());
        s.add_idx(field, field_value_fn, store);
        s
    }

    pub fn add_idx(
        &mut self,
        field: &'i str,
        field_value_fn: FieldValueFn<'i, T>,
        store: impl FilterableStore<'k, 'i> + 'i,
    ) {
        self.0
            .push(FieldStore::new(field, field_value_fn, Box::new(store)))
    }

    pub fn get_idx(&self, idx_name: &'k str) -> &FieldStore<'k, 'i, T> {
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
    fn person_indices() -> Result {
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

        assert_eq!([1], *indices.query(eq("pk", 41)).exec()?);
        assert_eq!([0], *indices.query(eq("pk", 3)).exec()?);
        assert!(indices.query(eq("pk", 101)).exec()?.is_empty());

        let r = indices.query(eq("second", 7)).exec()?;
        assert_eq!(*r, [0, 1]);

        let r = indices.query(eq("second", 3)).or(eq("second", 7)).exec()?;
        assert_eq!(*r, [0, 1]);

        let r = indices.query(eq("name", "Jasmin")).exec()?;
        assert_eq!(*r, [0]);

        let r = indices
            .query(eq("name", "Jasmin"))
            .or(eq("name", "Mario"))
            .exec()?;
        assert_eq!(*r, [0, 1]);

        let r = indices.query(eq("gender", Gender::Male)).exec()?;
        assert_eq!(*r, [1, 2]);

        let r = indices.query(eq("gender", Gender::Female)).exec()?;
        assert_eq!(*r, [0]);

        Ok(())
    }

    struct Idxs<'k, 's>(
        Box<dyn FilterableStore<'k, 's> + 's>,
        Box<dyn FilterableStore<'k, 's> + 's>,
    );

    impl<'k, 's> Filterable<'k> for Idxs<'k, 's> {
        fn filter(&self, p: Predicate<'k>) -> Result<Cow<[usize]>> {
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

        let r = idxs.query(1).and("a").exec()?;
        assert_eq!(*r, [1]);

        let r = idxs.query("z").or(1).and("a").exec()?;
        // = "z" or = 1 and = "a" => (= 1 and "a") or "z"
        assert_eq!(*r, [0, 1]);

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
