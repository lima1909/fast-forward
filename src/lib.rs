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

/// Empty array of `Idx`
pub const EMPTY_IDXS: &[Idx] = &[];

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

pub struct Iter<'i, T> {
    pos: usize,
    idxs: Cow<'i, [Idx]>,
    data: &'i [T],
}

impl<'i, T> Iter<'i, T> {
    fn new(idxs: Cow<'i, [Idx]>, data: &'i [T]) -> Self {
        Self { pos: 0, idxs, data }
    }
}

impl<'i, T> Iterator for Iter<'i, T> {
    type Item = &'i T;

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.idxs.get(self.pos)?;
        self.pos += 1;
        Some(&self.data[*i])
    }
}

pub trait IndexedList<T>: AsRef<[T]> {
    /// **Importand:** if an `Idx` is not valid (inside the borders), then this mehtod panics (OutOfBound).
    #[inline]
    fn filter<'i>(&'i self, idxs: Cow<'i, [Idx]>) -> Iter<'i, T> {
        Iter::new(idxs, self.as_ref())
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

    pub fn insert<K>(&mut self, v: T)
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
        index::{map::MapIndex, uint::UIntIndex, Equals},
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
        l.insert(Car::new(2, "BMW"));
        l.insert(Car::new(5, "Audi"));
        l.insert(Car::new(2, "VW"));
        l.insert(Car::new(99, "Porsche"));

        let r = l.filter(l.eq(2)).collect::<Vec<_>>();
        assert_eq!(vec![&Car::new(2, "BMW"), &Car::new(2, "VW")], r);

        let mut r = l.filter(l.eq_iter(2..6));
        assert_eq!(Some(&Car::new(2, "BMW")), r.next());
        assert_eq!(Some(&Car::new(5, "Audi")), r.next());
        assert_eq!(Some(&Car::new(2, "VW")), r.next());
        assert_eq!(None, r.next());

        let r = l
            .filter(query(l.eq(2)).or(l.eq(100)).exec())
            .collect::<Vec<_>>();
        assert_eq!(&[&Car::new(2, "BMW"), &Car::new(2, "VW")], &r[..]);
    }

    #[test]
    fn one_indexed_list_string() {
        let mut l = OneIndexedList::new(Car::name, MapIndex::default());
        l.insert(Car::new(2, "BMW"));
        l.insert(Car::new(5, "Audi"));
        l.insert(Car::new(2, "VW"));
        l.insert(Car::new(99, "Porsche"));

        let r: Vec<&Car> = l.filter(l.eq(&"VW".into())).collect();
        assert_eq!(vec![&Car::new(2, "VW")], r);

        let r: Vec<&Car> = l
            .filter(l.eq_iter([&"VW".into(), &"Audi".into(), &"BMW".into()]))
            .collect();
        assert_eq!(
            vec![
                &Car::new(2, "BMW"),
                &Car::new(5, "Audi"),
                &Car::new(2, "VW"),
            ],
            r
        );

        let r: Vec<&Car> = l
            .filter(query(l.eq(&"VW".into())).or(l.eq(&"Audi".into())).exec())
            .collect();
        assert_eq!(vec![&Car::new(5, "Audi"), &Car::new(2, "VW")], r)
    }

    #[test]
    fn fast() {
        let mut c = fast!(Car {
            id: UIntIndex,
            name.clone: MapIndex,
        });

        let c1 = Car {
            id: 4,
            _no_index: 8,
            name: "Foo".into(),
        };
        c.insert(&c1, 1);

        assert_eq!([1], *query(c.id.eq(4)).or(c.name.eq(&"Foo".into())).exec());
    }

    #[derive(Debug, Clone, Copy, Default)]
    enum Gender {
        Male,
        Female,
        #[default]
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
        multi: u16,
        name: String,
        gender: Gender,
    }

    impl Person {
        fn new(pk: usize, multi: u16, name: &str, gender: Gender) -> Self {
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
        use Gender::*;

        let mut p = fast!(
                Person as FastPerson {
                    pk: UIntIndex,
                    multi: UIntIndex<u16>,
                    name.clone: MapIndex,
                    gender.into: UIntIndex<Gender>,
                }
        );

        let persons = vec![
            Person::new(3, 7, "Jasmin", Female),
            Person::new(41, 7, "Mario", Male),
            Person::new(111, 234, "Paul", Male),
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

        let r = query(p.name.eq(&"Jasmin".into())).exec();
        assert_eq!(*r, [0]);

        let r = query(p.name.eq(&"Jasmin".into()))
            .or(p.name.eq(&"Mario".into()))
            .exec();
        assert_eq!(*r, [0, 1]);

        let r = query(p.gender.eq(Male)).exec();
        assert_eq!(*r, [1, 2]);

        let r = query(p.gender.eq(Female)).exec();
        assert_eq!(*r, [0]);
    }

    #[test]
    fn different_idxs() {
        use crate::index::Store;
        use Gender::*;

        let p = Person::new(0, 0, "Julia", Female);

        let mut gender = UIntIndex::<Gender>::default();
        gender.insert(p.gender, 1);
        gender.insert(Male, 2);
        gender.insert(None, 0);

        let mut name = MapIndex::default();
        name.insert(p.name.as_ref(), 1);
        name.insert("b", 2);
        name.insert("z", 0);

        let r = query(gender.eq(Female)).and(name.eq(&"Julia")).exec();
        assert_eq!(*r, [1]);

        let r = query(name.eq(&"z"))
            .or(gender.eq(Female))
            .and(name.eq(&"Julia"))
            .exec();
        // = "z" or = 1 and = "a" => (= 1 and "a") or "z"
        assert_eq!(*r, [0, 1]);
    }
}
