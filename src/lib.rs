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

use std::borrow::Cow;

/// `Idx` is the index/position in a List ([`std::vec::Vec`]).
pub type Idx = usize;

/// Empty array of `Idx`
pub const EMPTY_IDXS: &[Idx] = &[];

/// Create an `IndexedList` on a given `struct`.
///
/// ## Example for struct Person
///
/// ```not_run
/// struct Person {
///     id: usize,
///     name: String,
/// }
///
/// let fast_persons = fast!(
///     FastPersonList => Person {
///         id:   UIntIndex => id,
///         name: MapIndex  => name.clone,
///     }
/// );
/// ```

#[macro_export]
macro_rules! fast {
    (
        $fast:ident on $strukt:ident {
            $( $fast_field:tt: $typ:ty => $strukt_field:tt $(.$func:ident)? ), + $(,)*
        }
    ) => {

        {

        /// Container-struct for all indices.
        #[derive(Default)]
        struct $fast {
            data: Vec<$strukt>,
            $(
                $fast_field: $typ,
            )+
        }

        /// Insert in all indices-stores the `Key` and the `Index`.
        impl $fast {
            fn insert(&mut self, s: $strukt)  {
                use $crate::index::Store;

                $(
                    self.$fast_field.insert(s.$strukt_field$(.$func())?, self.data.len());
                )+
                self.data.push(s);

            }
        }

        impl $crate::IndexedList<$strukt> for $fast {}

        impl AsRef<[$strukt]> for $fast {
            fn as_ref(&self) -> &[$strukt] {
                &self.data
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

#[cfg(test)]
mod tests {
    use crate::{
        fast,
        index::{map::MapIndex, uint::UIntIndex, Equals},
        query::query,
        IndexedList,
    };

    #[derive(Debug, Eq, PartialEq)]
    struct Car(usize, String);

    #[test]
    fn one_indexed_list_idx() {
        let mut cars = fast!(Cars on Car {id: UIntIndex => 0});
        cars.insert(Car(2, "BMW".into()));
        cars.insert(Car(5, "Audi".into()));
        cars.insert(Car(2, "VW".into()));
        cars.insert(Car(99, "Porsche".into()));

        let r = cars.filter(cars.id.eq(2)).collect::<Vec<_>>();
        assert_eq!(vec![&Car(2, "BMW".into()), &Car(2, "VW".into())], r);

        let mut r = cars.filter(cars.id.eq_iter(2..6));
        assert_eq!(Some(&Car(2, "BMW".into())), r.next());
        assert_eq!(Some(&Car(5, "Audi".into())), r.next());
        assert_eq!(Some(&Car(2, "VW".into())), r.next());
        assert_eq!(None, r.next());

        let r = cars
            .filter(query(cars.id.eq(2)).or(cars.id.eq(100)).exec())
            .collect::<Vec<_>>();
        assert_eq!(&[&Car(2, "BMW".into()), &Car(2, "VW".into())], &r[..]);
    }

    #[test]
    fn one_indexed_list_string() {
        let mut cars = fast!(Cars on Car {name: MapIndex => 1.clone});
        cars.insert(Car(2, "BMW".into()));
        cars.insert(Car(5, "Audi".into()));
        cars.insert(Car(2, "VW".into()));
        cars.insert(Car(99, "Porsche".into()));

        let r: Vec<&Car> = cars.filter(cars.name.eq(&"VW".into())).collect();
        assert_eq!(vec![&Car(2, "VW".into())], r);

        let r: Vec<&Car> = cars
            .filter(
                cars.name
                    .eq_iter([&"VW".into(), &"Audi".into(), &"BMW".into()]),
            )
            .collect();
        assert_eq!(
            vec![
                &Car(2, "BMW".into()),
                &Car(5, "Audi".into()),
                &Car(2, "VW".into()),
            ],
            r
        );

        let r: Vec<&Car> = cars
            .filter(
                query(cars.name.eq(&"VW".into()))
                    .or(cars.name.eq(&"Audi".into()))
                    .exec(),
            )
            .collect();
        assert_eq!(vec![&Car(5, "Audi".into()), &Car(2, "VW".into())], r)
    }

    #[test]
    fn fast() {
        let mut fast_cars = fast!(
                FastCars on Car {
                    id:     UIntIndex       => 0,
                    id_map: MapIndex<usize> => 0,
                    name:   MapIndex        => 1.clone,
                }
        );
        fast_cars.insert(Car(1, "Mercedes".into()));
        fast_cars.insert(Car(4, "Porsche".into()));

        assert_eq!([0], *query(fast_cars.id_map.eq(&1)).exec());
        assert_eq!(
            [1],
            *query(fast_cars.id.eq(4))
                .or(fast_cars.name.eq(&"Porsche".into()))
                .exec()
        );

        let r = fast_cars.filter(
            query(fast_cars.id.eq(4))
                .and(fast_cars.name.eq(&"Porsche".into()))
                .exec(),
        );
        assert_eq!(vec![&Car(4, "Porsche".into())], r.collect::<Vec<_>>())
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

        // CREATE INDEX index1 ON schema1.table1 (column1);
        let mut persons = fast!(
                Persons on Person {
                    pk:     UIntIndex         => pk,
                    multi:  UIntIndex<u16>    => multi,
                    name:   MapIndex          => name.clone,
                    gender: UIntIndex<Gender> => gender.into,
                }
        );

        persons.insert(Person::new(3, 7, "Jasmin", Female));
        persons.insert(Person::new(41, 7, "Mario", Male));
        persons.insert(Person::new(111, 234, "Paul", Male));

        assert_eq!([1], *query(persons.pk.eq(41)).exec());
        assert_eq!([0], *query(persons.pk.eq(3)).exec());
        assert!(query(persons.pk.eq(101)).exec().is_empty());

        let r = query(persons.multi.eq(7)).exec();
        assert_eq!(*r, [0, 1]);

        let r = query(persons.multi.eq(3)).or(persons.multi.eq(7)).exec();
        assert_eq!(*r, [0, 1]);

        let r = query(persons.name.eq(&"Jasmin".into())).exec();
        assert_eq!(*r, [0]);

        let r = query(persons.name.eq(&"Jasmin".into()))
            .or(persons.name.eq(&"Mario".into()))
            .exec();
        assert_eq!(*r, [0, 1]);

        let r = query(persons.gender.eq(Male)).exec();
        assert_eq!(*r, [1, 2]);

        let r = query(persons.gender.eq(Female)).exec();
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
