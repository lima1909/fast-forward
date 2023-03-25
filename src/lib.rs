//! **Fast-Forward** is a library for filtering items in a (large) list, _faster_ than an `Iterator` ([`std::iter::Iterator::filter`]).
//!
//! This _faster_ is achieved  by using `Indices`. This means, it does not have to touch and compare every item in the list.
//!
//! An Index has two parts, a `Key` (item to searching for) and a position (the index in the list) [`Idx`].
//!
//! ## A simple Example:
//!
//! ```text
//! let _list_with_names = vec!["Paul", "Jon", "Inge", "Paul", ...];
//! ```
//!
//! Index `Map(name, idx's)`:
//!
//! ```text
//!  Key     | Idx
//! ---------------
//!  "Paul"  | 0, 3
//!  "Jon"   | 1
//!  "Inge"  | 2
//!   ...    | ...
//! ```
//!
//! To Find the `Key`: "Jon" with the `operation = equals` is only one step necessary.
//!
pub mod error;
pub mod index;
pub mod query;

pub use query::query;

// Default Result for index with the Ok(T) value or en [`error::Error`].
// pub type Result<T = ()> = std::result::Result<T, error::Error>;

/// `Idx` is the index/position in a List ([`std::vec::Vec`]).
pub type Idx = usize;

#[macro_export]
macro_rules! fast {
    (   $strukt:ident$(<$lt:lifetime>)?
        {
            $( $fast_field:ident $(.$func:ident)?: $typ:ty ), + $(,)*
        }
    ) => {
        fast!($strukt$(<$lt>)? as Fast { $( $fast_field $(.$func)?: $typ ), + })
    };

    (   $strukt:ident$(<$lt:lifetime>)? as $fast:ident
        {
            $( $fast_field:ident $(.$func:ident)?: $typ:ty ), + $(,)*
        }

    ) => {

        {

        /// Container-struct for all indices.
        #[derive(Default)]
        struct $fast$(<$lt>)? {
            $(
                $fast_field: $typ,
            )+
        }

        /// Insert in all indices-stores the `Key` and the `Index`.
        impl$(<$lt>)? $fast$(<$lt>)? {
            fn insert(&mut self, s: &$($lt)? $strukt, idx: $crate::Idx)  {
                use $crate::index::Store;

                $(
                    self.$fast_field.insert(s.$fast_field$(.$func())?, idx);
                )+

            }
        }

        $fast::default()

        }

    };
}

#[cfg(test)]
mod tests {

    struct Car {
        id: usize,
        _multi: usize,
        name: String,
    }

    #[test]
    fn fast() {
        use crate::index::{map::StrMapIndex, uint::UIntIndex};
        use crate::query;

        let mut c = fast!(
                Car<'p> {
                    id: UIntIndex,
                    name.as_ref: StrMapIndex<'p>,
                }
        );

        let c1 = Car {
            id: 4,
            _multi: 8,
            name: "Foo".into(),
        };
        c.insert(&c1, 1);

        assert_eq!([1], *query(c.id.eq(4)).or(c.name.eq("Foo")).exec());
    }

    use crate::{
        fast,
        index::{map::StrMapIndex, uint::UIntIndex},
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
                    pk: UIntIndex,
                    multi: UIntIndex,
                    name.as_ref: StrMapIndex<'p>,
                    gender.into: UIntIndex,
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
            .for_each(|(i, person)| p.insert(person, i));

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
        use crate::index::Store;

        let mut pk = UIntIndex::default();
        pk.insert(1, 1);
        pk.insert(2, 2);
        pk.insert(99, 0);

        let p = Person::new(3, 7, "a", Gender::None);
        let mut name = StrMapIndex::default();
        name.insert(&p.name, 1);
        name.insert("b", 2);
        name.insert("z", 0);

        let r = query(pk.eq(1)).and(name.eq("a")).exec();
        assert_eq!(*r, [1]);

        let r = query(name.eq("z")).or(pk.eq(1)).and(name.eq("a")).exec();
        // = "z" or = 1 and = "a" => (= 1 and "a") or "z"
        assert_eq!(*r, [0, 1]);
    }
}
