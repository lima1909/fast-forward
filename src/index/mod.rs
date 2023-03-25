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

pub use idx::{Index, Multi, Unique};

use crate::{Idx, Result};

/// A Store is a mapping from a given `Key` to one or many `Indices`.
pub trait Store<K> {
    /// Insert an `Key` for a given `Index`.
    ///
    /// Before:
    ///     Female | 3,4
    /// `Insert: (Male, 2)`
    /// After:
    ///     Male   | 2
    ///     Female | 3,4
    ///
    /// OR (if the `Key` already exist):
    ///
    /// Before:
    ///     Female | 3,4
    /// `Insert: (Female, 2)`
    /// After:
    ///     Female | 2,3,4
    ///
    fn insert(&mut self, key: K, idx: Idx) -> Result;

    /// Update means: `Key` changed, but `Index` stays the same
    ///
    /// Before:
    ///     Male   | 1,2,5  
    ///     Female | 3,4
    /// `Update: (Male, 2, Female)`
    /// After:
    ///     Male   | 1,5
    ///     Female | 2,3,4
    ///
    /// If the old `Key` not exist, then is it a insert with the new `Key`:
    ///
    /// Before:
    ///     Female | 3,4
    /// `Update: (Male, 2, Female)`
    /// After:
    ///     Female | 2,3,4

    fn update(&mut self, _old_key: K, _idx: Idx, _new_key: K) -> Result {
        Ok(())
    }

    /// Delete means: if an `Key` has more than one `Index`, then remove only this `Index`:
    ///
    /// Before:
    ///     Male   | 1,2,5  
    ///     Female | 3,4
    /// `Delete: Male: 2`
    /// After:
    ///     Male   | 1,5
    ///     Female | 3,4
    ///
    /// otherwise (`Key` has exact one `Index`), then remove complete row (`Key` and `Index`).
    ///
    /// Before:
    ///     Male   | 2
    ///     Female | 3,4
    /// `Delete: Male: 2`
    /// After:
    ///     Female | 3,4
    ///
    /// If the `Key` not exist, then is `delete`ignored:
    ///
    /// Before:
    ///     Female | 3,4
    /// `Delete: Male: 2`
    /// After:
    ///     Female | 3,4
    ///
    fn delete(&mut self, _key: K, _idx: Idx) -> Result {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        fast,
        index::{
            map::UniqueStrIdx,
            uint::{MultiUintIdx, PkUintIdx},
            Store,
        },
        query,
    };

    #[derive(Debug, Clone, Copy)]
    enum Gender {
        Male,
        Female,
        None,
    }

    impl From<Gender> for usize {
        fn from(g: Gender) -> Self {
            match g {
                Gender::None => 0,
                Gender::Male => 1,
                Gender::Female => 2,
            }
        }
    }

    #[derive(Debug)]
    struct Person {
        pk: usize,
        multi: usize,
        name: String,
        gender: Gender,
    }

    impl Person {
        fn new(pk: usize, multi: usize, name: &str, gender: Gender) -> Self {
            Self {
                pk,
                multi,
                name: name.to_string(),
                gender,
            }
        }
    }

    #[test]
    fn person_indices() {
        let mut p = fast!(
                Person<'p> as FastPerson {
                    pk: PkUintIdx,
                    multi: MultiUintIdx,
                    name.as_ref: UniqueStrIdx<'p>,
                    gender.into: MultiUintIdx,
                }
        );

        let persons = vec![
            Person::new(3, 7, "Jasmin", Gender::Female),
            Person::new(41, 7, "Mario", Gender::Male),
            Person::new(111, 234, "Paul", Gender::Male),
        ];

        persons
            .iter()
            .enumerate()
            .for_each(|(i, person)| p.insert(person, i).unwrap());

        assert_eq!([1], *query(p.pk.eq(41)).exec());
        assert_eq!([0], *query(p.pk.eq(3)).exec());
        assert!(query(p.pk.eq(101)).exec().is_empty());

        let r = query(p.multi.eq(7)).exec();
        assert_eq!(*r, [0, 1]);

        let r = query(p.multi.eq(3)).or(p.multi.eq(7)).exec();
        assert_eq!(*r, [0, 1]);

        let r = query(p.name.eq("Jasmin")).exec();
        assert_eq!(*r, [0]);

        let r = query(p.name.eq("Jasmin")).or(p.name.eq("Mario")).exec();
        assert_eq!(*r, [0, 1]);

        let r = query(p.gender.eq(Gender::Male.into())).exec();
        assert_eq!(*r, [1, 2]);

        let r = query(p.gender.eq(Gender::Female.into())).exec();
        assert_eq!(*r, [0]);
    }

    #[test]
    fn different_idxs() {
        let mut pk = PkUintIdx::default();
        pk.insert(1, 1).unwrap();
        pk.insert(2, 2).unwrap();
        pk.insert(99, 0).unwrap();

        let p = Person::new(3, 7, "a", Gender::None);
        let mut name = UniqueStrIdx::default();
        name.insert(&p.name, 1).unwrap();
        name.insert("b", 2).unwrap();
        name.insert("z", 0).unwrap();

        let r = query(pk.eq(1)).and(name.eq("a")).exec();
        assert_eq!(*r, [1]);

        let r = query(name.eq("z")).or(pk.eq(1)).and(name.eq("a")).exec();
        // = "z" or = 1 and = "a" => (= 1 and "a") or "z"
        assert_eq!(*r, [0, 1]);
    }
}
