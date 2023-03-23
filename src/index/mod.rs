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

/// A Store for a mapping from a given Key to one or many Indices.
pub trait Store<K> {
    /// Insert all indices for a given `Key`.
    fn insert(&mut self, k: K, i: Idx) -> Result;
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
        // gender: Gender,
    }

    impl Person {
        fn new(pk: usize, multi: usize, name: &str, _gender: Gender) -> Self {
            Self {
                pk,
                multi,
                name: name.to_string(),
                // gender,
            }
        }
    }

    #[test]
    fn person_indices() {
        let persons = vec![
            Person::new(3, 7, "Jasmin", Gender::Female),
            Person::new(41, 7, "Mario", Gender::Male),
            Person::new(111, 234, "Paul", Gender::Male),
        ];

        let mut p = fast!(
                Person<'p> as FastPerson {
                    pk: PkUintIdx,
                    multi: MultiUintIdx,
                    name: UniqueStrIdx<'p> => &,
                    // gender: UIntVecIndex<Multi>,
                }
        );

        p.insert(&persons[0], 0).unwrap();
        p.insert(&persons[1], 1).unwrap();
        p.insert(&persons[2], 2).unwrap();

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

        // let r = query(p.gender.eq(Gender::Male.into())).exec();
        // assert_eq!(*r, [1, 2]);

        // let r = query(p.gender.eq(Gender::Female.into())).exec();
        // assert_eq!(*r, [0]);
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
