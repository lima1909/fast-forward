//! **Fast-Forward** is a library for filtering items in a (large) list, _faster_ than an `Iterator` ([`std::iter::Iterator::filter`]).
//! It is not a replacement of the `Iterator`, but an addition.
//!
//! This _faster_ is achieved  by using `Indices`. This means, it does not have to touch and compare every item in the list.
//!
//! An `Index` has two parts, a `Key` (item to searching for) and a `Position` (the index) in the list.
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
//! To Find the `Key`: "Jon" with the `operation equals` is only one step necessary.
//!

use std::borrow::Cow;

pub mod index;
pub mod list;
pub mod query;

/// Empty array of `Idx`
pub const EMPTY_IDXS: &[usize] = &[];

/// `Filterable` means, that you get an `Iterator` over all `Items` which exists for a given list of indices.
pub trait Filterable {
    type Item;

    /// Returns `Some(Item)` from the given index (position) if it exist, otherwise `None`
    fn item(&self, index: usize) -> Option<&Self::Item>;

    /// Returns a `Iterator` over all `Items` with the given index list.
    fn filter<'i>(&'i self, indices: Cow<'i, [usize]>) -> Filter<'i, Self>
    where
        Self: Sized,
    {
        Filter::new(self, indices)
    }
}

pub struct Filter<'i, F: Filterable> {
    pos: usize,
    list: &'i F,
    indices: Cow<'i, [usize]>,
}

impl<'i, F> Filter<'i, F>
where
    F: Filterable,
{
    pub const fn new(list: &'i F, indices: Cow<'i, [usize]>) -> Self {
        Self {
            pos: 0,
            list,
            indices,
        }
    }
}

impl<'i, F> Iterator for Filter<'i, F>
where
    F: Filterable,
{
    type Item = &'i F::Item;

    fn next(&mut self) -> Option<Self::Item> {
        while self.pos < self.indices.len() {
            let idx = self.indices[self.pos];
            self.pos += 1;
            match self.list.item(idx) {
                Some(item) => return Some(item),
                // ignore deleted items
                None => continue,
            }
        }
        None
    }
}

/// This `macro` is not a solution, it is more an POC (proof of concept)!
/// The Problem with this macro is the visibility. This means, it can not hide internal fields,
/// like the `_items_` Vec, for example. But it illustrate the idea behind `fast forward`.
///
/// Create an `Indexed List` on a given `struct`.
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
        $fast:ident on $item:ident {
            $( $store:tt: $store_type:ty => $item_field:tt $(.$item_field_func:ident)? ), + $(,)*
        }
    ) => {

        {

        /// Container-struct for all indices.
        #[derive(Default)]
        struct $fast {
            $(
                $store: $store_type,
            )+
            _items_: $crate::list::List<$item>,
        }

        ///
        impl $fast {

            /// Insert the given item.
            ///
            #[allow(dead_code)]
            fn insert(&mut self, item: $item) -> usize {
                use $crate::index::Store;

                self._items_.insert(item, |it: &$item, pos: usize| {
                    $(
                        self.$store.insert(
                                    it.$item_field$(.$item_field_func())?,
                                    pos
                                    );
                    )+
                })

            }

            /// Update the item on the given position.
            ///
            /// # Panics
            ///
            /// Panics if the pos is out of bound.
            ///
            #[allow(dead_code)]
            fn update<F>(&mut self, pos: usize, update_fn: F) -> bool where F: Fn(&$item)-> $item {
                use $crate::index::Store;

                self._items_.update(pos, update_fn, |old: &$item, pos: usize, new: &$item| {
                    $(
                        self.$store.update(
                                    old.$item_field$(.$item_field_func())?,
                                    pos,
                                    new.$item_field$(.$item_field_func())?
                                    );
                    )+
                })
            }

            /// Delete the item on the given position.
            ///
            /// # Panics
            ///
            /// Panics if the pos is out of bound.
            ///
            #[allow(dead_code)]
            fn delete(&mut self, pos: usize) -> &$item{
                use $crate::index::Store;

                self._items_.delete(pos, |it: &$item, pos: usize| {
                    $(
                        self.$store.delete(
                                    it.$item_field$(.$item_field_func())?,
                                    pos
                                    );
                    )+
                })
            }

            /// Create an Iterator for the given Filter.
            ///
            /// # Panics
            ///
            /// Panics if the positions are out of bound.
            ///
            #[allow(dead_code)]
            fn filter<'i>(&'i self, filter: std::borrow::Cow<'i, [usize]>) -> $crate::Filter<'i, $crate::list::List<$item>> {
                use $crate::Filterable;

                self._items_.filter(filter)
            }

            #[allow(dead_code)]
            fn iter(&self) -> $crate::list::Iter<'_, $item> {
                self._items_.iter()
            }

            $(
                /// Create and get a Filter for the Store
                #[allow(dead_code)]
                fn $store(&self) -> <$store_type as $crate::index::Store>::Filter<'_, $item, $crate::list::List<$item>> {
                    use $crate::index::Store;

                    self.$store.create_filter(&self._items_)
                }
            )+
        }


        $fast::default()

        }

    };
}

#[cfg(test)]
mod tests {
    use crate::{
        fast,
        index::{map::MapIndex, uint::UIntIndex, Equals},
        query::query,
    };

    #[derive(Debug, Eq, PartialEq, Clone)]
    struct Car(usize, String);

    #[test]
    fn one_indexed_list_filter() {
        let mut cars = fast!(Cars on Car {id: UIntIndex => 0});
        cars.insert(Car(2, "BMW".into()));
        cars.insert(Car(5, "Audi".into()));
        cars.insert(Car(2, "VW".into()));
        cars.insert(Car(99, "Porsche".into()));

        let id_filter = cars.id();
        let r = id_filter.get(2).collect::<Vec<_>>();
        assert_eq!(vec![&Car(2, "BMW".into()), &Car(2, "VW".into())], r);

        let mut it = id_filter.get(5);
        assert_eq!(it.next(), Some(&Car(5, "Audi".into())));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn one_indexed_list_delete_item() {
        let mut cars = fast!(Cars on Car {id: UIntIndex => 0});
        cars.insert(Car(0, "Porsche".into()));
        cars.insert(Car(1, "BMW".into()));
        cars.insert(Car(2, "Porsche".into()));
        cars.insert(Car(3, "Audi".into()));
        cars.insert(Car(4, "VW".into()));
        cars.insert(Car(5, "VW".into()));

        let r = cars.iter().collect::<Vec<_>>();
        assert_eq!(
            vec![
                &Car(0, "Porsche".into()),
                &Car(1, "BMW".into()),
                &Car(2, "Porsche".into()),
                &Car(3, "Audi".into()),
                &Car(4, "VW".into()),
                &Car(5, "VW".into())
            ],
            r
        );

        cars.delete(3);

        let r = cars.iter().collect::<Vec<_>>();
        assert_eq!(
            vec![
                &Car(0, "Porsche".into()),
                &Car(1, "BMW".into()),
                &Car(2, "Porsche".into()),
                &Car(4, "VW".into()),
                &Car(5, "VW".into())
            ],
            r
        );

        // idx [1,3,5]
        // del [3]
        let r = cars.filter(cars.id.eq_iter([1, 3, 5])).collect::<Vec<_>>();
        assert_eq!(vec![&Car(1, "BMW".into()), &Car(5, "VW".into())], r);
    }

    #[test]
    fn one_indexed_list_idx() {
        let mut cars = fast!(Cars on Car {id: UIntIndex => 0});
        cars.insert(Car(2, "BMW".into()));
        cars.insert(Car(5, "Audi".into()));
        cars.insert(Car(2, "VW".into()));
        cars.insert(Car(99, "Porsche".into()));

        // simple equals filter
        let r = cars.filter(cars.id.eq(2)).collect::<Vec<_>>();
        assert_eq!(vec![&Car(2, "BMW".into()), &Car(2, "VW".into())], r);

        // many/iter equals filter
        let mut r = cars.filter(cars.id.eq_iter(2..6));
        assert_eq!(Some(&Car(2, "BMW".into())), r.next());
        assert_eq!(Some(&Car(5, "Audi".into())), r.next());
        assert_eq!(Some(&Car(2, "VW".into())), r.next());
        assert_eq!(None, r.next());

        // or equals query
        let r = cars
            .filter(query(cars.id.eq(2)).or(cars.id.eq(100)).exec())
            .collect::<Vec<_>>();
        assert_eq!(&[&Car(2, "BMW".into()), &Car(2, "VW".into())], &r[..]);

        // update one Car
        assert_eq!(None, cars.filter(cars.id.eq(100)).next());
        cars.update(cars.id.eq(99)[0], |c: &Car| Car(c.0 + 1, c.1.clone()));
        let r = cars.filter(cars.id.eq(100)).collect::<Vec<_>>();
        assert_eq!(vec![&Car(100, "Porsche".into())], r);

        // remove one Car
        assert!(cars.filter(cars.id.eq(100)).next().is_some());
        cars.delete(cars.id.eq(100)[0]);
        assert_eq!(None, cars.filter(cars.id.eq(100)).next());
    }

    #[test]
    fn one_indexed_list_idx_min_max() {
        let mut cars = fast!(Cars on Car {id: UIntIndex => 0});
        cars.insert(Car(2, "BMW".into()));
        cars.insert(Car(5, "Audi".into()));
        cars.insert(Car(2, "VW".into()));
        cars.insert(Car(99, "Porsche".into()));

        // simple equals filter
        let r = cars.filter(cars.id.eq(2)).collect::<Vec<_>>();
        assert_eq!(vec![&Car(2, "BMW".into()), &Car(2, "VW".into())], r);

        // min and max
        assert_eq!(2, cars.id.min());
        assert_eq!(99, cars.id.max());
    }

    #[test]
    fn one_indexed_list_string() {
        let mut cars = fast!(Cars on Car {name: MapIndex => 1.to_lowercase});
        cars.insert(Car(2, "BMW".into()));
        cars.insert(Car(5, "Audi".into()));
        cars.insert(Car(2, "VW".into()));
        cars.insert(Car(99, "Porsche".into()));

        // simple equals filter
        let r: Vec<&Car> = cars.filter(cars.name.eq(&"vw".into())).collect();
        assert_eq!(vec![&Car(2, "VW".into())], r);

        // many/iter equals filter
        let r: Vec<&Car> = cars
            .filter(
                cars.name
                    .eq_iter([&"vw".into(), &"audi".into(), &"bmw".into()]),
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

        // or equals query
        let r: Vec<&Car> = cars
            .filter(
                query(cars.name.eq(&"vw".into()))
                    .or(cars.name.eq(&"audi".into()))
                    .exec(),
            )
            .collect();
        assert_eq!(vec![&Car(5, "Audi".into()), &Car(2, "VW".into())], r);

        // update one Car
        assert_eq!(None, cars.filter(cars.name.eq(&"mercedes".into())).next());
        cars.update(cars.name.eq(&"porsche".into())[0], |c: &Car| {
            Car(c.0, "Mercedes".into())
        });
        let r = cars
            .filter(cars.name.eq(&"mercedes".into()))
            .collect::<Vec<_>>();
        assert_eq!(vec![&Car(99, "Mercedes".into())], r);

        // remove one Car
        assert!(cars
            .filter(cars.name.eq(&"mercedes".into()))
            .next()
            .is_some());
        cars.delete(cars.name.eq(&"mercedes".into())[0]);
        assert_eq!(None, cars.filter(cars.name.eq(&"mercedes".into())).next());
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

    #[derive(Debug, Clone)]
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
