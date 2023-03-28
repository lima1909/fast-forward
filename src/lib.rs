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

use std::{borrow::Cow, ops::Deref};

/// `Idx` is the index/position in a List ([`std::vec::Vec`]).
pub type Idx = usize;

#[macro_export]
macro_rules! fast {
    (   $strukt:ident
        {
            $( $fast_field:ident $(.$func:ident)?: $typ:ty ), + $(,)*
        }
    ) => {
        fast!($strukt as Fast { $( $fast_field $(.$func)?: $typ ), + })
    };

    (   $strukt:ident as $fast:ident
        {
            $( $fast_field:ident $(.$func:ident)?: $typ:ty ), + $(,)*
        }

    ) => {

        {

        /// Container-struct for all indices.
        #[derive(Default)]
        struct $fast {
            $(
                $fast_field: $typ,
            )+
        }

        /// Insert in all indices-stores the `Key` and the `Index`.
        impl $fast {
            fn insert(&mut self, s: &$strukt, idx: $crate::Idx)  {
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

pub trait IndexedList<T>: AsRef<[T]> {
    /// **Importand:** if an `Idx` is not valid (inside the borders), then this mehtod panics (OutOfBound).
    #[inline]
    fn filter(&self, idxs: Cow<'_, [Idx]>) -> Vec<&T> {
        let inner = self.as_ref();
        idxs.iter().map(|i| &inner[*i]).collect()
    }
}

pub struct OneIndexedList<T, F, S> {
    inner: Vec<T>,
    get_id_fn: F,
    store: S,
}

impl<T, F, S> OneIndexedList<T, F, S> {
    pub fn new(f: F, store: S) -> Self {
        Self {
            inner: vec![],
            get_id_fn: f,
            store,
        }
    }

    pub fn push<K>(&mut self, v: T)
    where
        S: crate::index::Store<K>,
        F: Fn(&T) -> K,
    {
        self.store.insert((self.get_id_fn)(&v), self.inner.len());
        self.inner.push(v);
    }
}

impl<T, F, S> IndexedList<T> for OneIndexedList<T, F, S> {}

impl<T, F, S> AsRef<[T]> for OneIndexedList<T, F, S> {
    fn as_ref(&self) -> &[T] {
        &self.inner
    }
}

impl<T, F, S> Deref for OneIndexedList<T, F, S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.store
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        fast,
        index::{map::StrMapIndex, uint::UIntIndex},
        query::query,
        IndexedList, OneIndexedList,
    };

    #[derive(Debug, Eq, PartialEq)]
    struct Car {
        id: usize,
        _no_index: usize,
        name: String,
    }

    impl Car {
        fn new(id: usize, name: &str) -> Self {
            Self {
                id,
                _no_index: 0,
                name: name.into(),
            }
        }

        fn id(&self) -> usize {
            self.id
        }

        fn name(&self) -> String {
            self.name.clone()
        }
    }

    #[test]
    fn one_indexed_list_idx() {
        let mut l = OneIndexedList::new(Car::id, UIntIndex::default());
        l.push(Car::new(2, "BMW"));
        l.push(Car::new(5, "Audi"));
        l.push(Car::new(2, "VW"));
        l.push(Car::new(99, "Porsche"));

        let r = l.filter(l.eq(2));
        assert_eq!(&[&Car::new(2, "BMW"), &Car::new(2, "VW")], &r[..]);

        let r = l.filter(query(l.eq(2)).or(l.eq(100)).exec());
        assert_eq!(&[&Car::new(2, "BMW"), &Car::new(2, "VW")], &r[..]);
    }

    #[test]
    fn one_indexed_list_string() {
        let mut l = OneIndexedList::new(Car::name, StrMapIndex::default());
        l.push(Car::new(2, "BMW"));
        l.push(Car::new(5, "Audi"));
        l.push(Car::new(2, "VW"));
        l.push(Car::new(99, "Porsche"));

        let r = l.filter(l.eq("VW"));
        assert_eq!(&[&Car::new(2, "VW")], &r[..]);

        let r = l.filter(query(l.eq("VW")).or(l.eq("Audi")).exec());
        assert_eq!(&[&Car::new(5, "Audi"), &Car::new(2, "VW")], &r[..])
    }

    #[test]
    fn fast() {
        let mut c = fast!(Car {
            id: UIntIndex,
            name.clone: StrMapIndex,
        });

        let c1 = Car {
            id: 4,
            _no_index: 8,
            name: "Foo".into(),
        };
        c.insert(&c1, 1);

        assert_eq!([1], *query(c.id.eq(4)).or(c.name.eq("Foo")).exec());
    }

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
                Person as FastPerson {
                    pk: UIntIndex,
                    multi: UIntIndex,
                    name.clone: StrMapIndex,
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
        name.insert(p.name, 1);
        name.insert("b".into(), 2);
        name.insert("z".into(), 0);

        let r = query(pk.eq(1)).and(name.eq("a")).exec();
        assert_eq!(*r, [1]);

        let r = query(name.eq("z")).or(pk.eq(1)).and(name.eq("a")).exec();
        // = "z" or = 1 and = "a" => (= 1 and "a") or "z"
        assert_eq!(*r, [0, 1]);
    }
}
