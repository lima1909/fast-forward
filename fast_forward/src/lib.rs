//! __Fast-Forward__ is a library for finding or filtering items in a (large) collection (Vec, Slice, Map, ...).
//! This means faster than an `iterator` or a `search algorithm`.
//! It is a wrapper, which extends the given collection with very fast find operations.
//! This wrapper is just as easy to use as the given (original) collection.
//!
//! This faster is achieved  by using `Indices`. This means, it does not have to touch and compare every item in the collection.
//!
//! An `Index` has two parts, a `Key` (item to searching for) and a `Position` (the index) in the collection.
//!
//! ## Example for an indexed read only List (ro::IList):
//!
//! ```
//! use fast_forward::{index::UIntIndex, collections::ro::IList};
//!
//! #[derive(Debug, PartialEq)]
//! pub struct Car(usize, String);
//!
//! // created an indexed List with the UIntIndex on the Car property 0.
//! let l = IList::<UIntIndex, _>::new(|c: &Car| c.0, vec![
//!                             Car(1, "BMW".into()),
//!                             Car(2, "VW".into())]);
//!
//! // idx method pointed to the Car.0 property Index and
//! // gives access to the `Retriever` object to handle queries,
//! // like: contains, get, filter.
//! assert!(l.idx().contains(&2));
//! assert!(!l.idx().contains(&2000));
//!
//! // get a Car with the ID = 2
//! assert_eq!(l.idx().get(&2).next(), Some(&Car(2, "VW".into())));
//!
//! // get many Cars with ID = 2 or 1
//! assert_eq!(
//!     l.idx().get_many([2, 1]).collect::<Vec<_>>(),
//!     vec![&Car(2, "VW".into()), &Car(1, "BMW".into())],
//! );
//!
//! // the same query with the filter-method
//! // (which has the disadvantage, that this need a allocation)
//! assert_eq!(
//!     l.idx().filter(|f| f.eq(&2) | f.eq(&1)).collect::<Vec<_>>(),
//!     vec![&Car(1, "BMW".into()), &Car(2, "VW".into())],
//! );
//!
//! // you can use the Vec methods too
//! assert_eq!(2, l.len());
//!
//! // or you can get MetaData like min and max Key value
//! use fast_forward::index::store::MetaData;
//!
//! assert_eq!(1, l.idx().meta().min_key());
//! assert_eq!(2, l.idx().meta().max_key());
//! ```
//!
//! All supported options for retrieve Items can you find by the [`crate::collections::Retriever`] struct.
//!
//! ## Example for a `View` of an indexed read only List (ro::IList):
//!
//! A `View` is like a database view. This means you get a subset of items, which you can see.
//! It is useful, if you don't want to give full read access to the complete collection.
//!
//! All details to [`crate::collections::Retriever::create_view()`]
//!
//! ```
//! use fast_forward::{index::MapIndex, collections::ro::IList};
//!
//! #[derive(Debug, PartialEq)]
//! pub struct Car(usize, String);
//!
//! // created an indexed List with the MapIndex on the Car property 1.
//! let l = IList::<MapIndex, _>::new(|c: &Car| c.1.clone(), vec![
//!                             Car(1, "BMW".into()),
//!                             Car(2, "VW".into()),
//!                             Car(3, "Audi".into())]);
//!
//! // create a view: only for Car Name = "BMW" 0r "Audi"
//! let view = l.idx().create_view(
//!       [String::from("BMW"), String::from("Audi")]);
//!
//! // Car with Name "VW" is NOT in the view
//! assert!(!view.contains(&String::from("VW")));
//!
//! // get the Care with the name "Audi"
//! assert_eq!(
//!     view.get(&String::from("Audi")).collect::<Vec<_>>(),
//!     vec![&Car(3, "Audi".into())],
//! );
//!
//! // the original list contains of course the Car with ID "VW"
//! assert!(l.idx().contains(&String::from("VW")));
//! ```
//!

pub mod collections;
pub mod index;

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
#[doc(hidden)]
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
            _items_: $crate::collections::rw::list_base::TriggerList<$item>,
        }

        ///
        impl $fast {

            /// Insert the given item.
            #[allow(dead_code)]
            fn insert(&mut self, item: $item) -> usize {
                use $crate::index::store::Store;

                self._items_.push(item, |it: &$item, pos: usize| {
                    $(
                        self.$store.insert(
                                    it.$item_field$(.$item_field_func())?,
                                    pos
                                    );
                    )+
                })

            }

            /// Update the item on the given position.
            #[allow(dead_code)]
            fn update<U>(&mut self, pos: usize, update: U) -> bool
            where
                U: Fn(&mut $item)
            {
                use $crate::index::store::{Filterable, Store};

                self._items_.get_mut(pos).map_or(false, |it| {
                    $(
                    let $store: <$store_type as Filterable>::Key = it.$item_field$(.$item_field_func())?;
                    )+
                    update(it);
                    $(
                    self.$store.update($store, pos, it.$item_field$(.$item_field_func())?);
                    )+
                    true
                })
            }

            /// Delete the item on the given position.
            #[allow(dead_code)]
            fn remove(&mut self, pos: usize) -> Option<$item> {
                use $crate::index::store::Store;
                use $crate::collections::rw::list_base::StoreOp;

                self._items_.remove(pos, |trigger, it, idx| match trigger {
                    StoreOp::Delete => { $( self.$store.delete(it.$item_field$(.$item_field_func())?, &idx); )+ }
                    StoreOp::Insert => { $( self.$store.insert(it.$item_field$(.$item_field_func())?, idx);  )+ }
                })
            }

            #[allow(dead_code)]
            fn iter(&self) -> impl Iterator<Item = &'_ $item> {
                self._items_.iter()
            }

            $(
                /// Create and get a Filter for the Store
                #[allow(dead_code)]
                fn $store(&self) -> $crate::collections::Retriever<'_, $store_type, Vec<$item>> {
                    $crate::collections::Retriever::new(&self.$store, &self._items_)
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
        index::{filter::Filter, map::MapIndex, store::Filterable, uint::UIntIndex},
    };

    #[derive(Debug, Eq, PartialEq)]
    struct Car(usize, String);

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

        assert_eq!(Car(3, "Audi".into()), cars.remove(3).unwrap());

        let r = cars.iter().collect::<Vec<_>>();
        assert_eq!(
            vec![
                &Car(0, "Porsche".into()),
                &Car(1, "BMW".into()),
                &Car(2, "Porsche".into()),
                &Car(5, "VW".into()),
                &Car(4, "VW".into())
            ],
            r
        );

        // idx [1,3,5]
        // del [3]
        let r = cars.id().get_many([1, 3, 5]).collect::<Vec<_>>();
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
        let r = cars.id().get(&2).collect::<Vec<_>>();
        assert_eq!(vec![&Car(2, "BMW".into()), &Car(2, "VW".into())], r);

        // many/iter equals filter
        {
            let mut r = cars.id().get_many(2..6);
            assert_eq!(Some(&Car(2, "BMW".into())), r.next());
            assert_eq!(Some(&Car(2, "VW".into())), r.next());
            assert_eq!(Some(&Car(5, "Audi".into())), r.next());
            assert_eq!(None, r.next());
        }

        // or equals query
        let r = cars
            .id()
            .filter(|f| f.eq(&2) | f.eq(&100))
            .collect::<Vec<_>>();
        assert_eq!(&[&Car(2, "BMW".into()), &Car(2, "VW".into())], &r[..]);

        // update one Car
        assert_eq!(None, cars.id().get(&100).next());
        cars.update(cars.id.get(&99)[0], |c| c.0 += 1);
        let r = cars.id().get(&100).collect::<Vec<_>>();
        assert_eq!(vec![&Car(100, "Porsche".into())], r);

        // remove one Car
        assert!(cars.id().get(&100).next().is_some());
        cars.remove(cars.id.get(&100)[0]);
        assert_eq!(None, cars.id().get(&100).next());
    }

    #[test]
    fn one_indexed_list_idx_min_max() {
        let mut cars = fast!(Cars on Car {id: UIntIndex => 0});
        cars.insert(Car(2, "BMW".into()));
        cars.insert(Car(5, "Audi".into()));
        cars.insert(Car(2, "VW".into()));
        cars.insert(Car(99, "Porsche".into()));

        // simple equals filter
        let r = cars.id().get(&2).collect::<Vec<_>>();
        assert_eq!(vec![&Car(2, "BMW".into()), &Car(2, "VW".into())], r);

        // min and max
        use crate::index::store::MetaData;
        assert_eq!(2, cars.id.meta().min_key());
        assert_eq!(99, cars.id.meta().max_key());
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

        let fid = Filter(&fast_cars.id);
        let fname = Filter(&fast_cars.name);

        assert_eq!([0], fast_cars.id_map.get(&1));
        assert_eq!([1], fid.eq(&4) | fname.eq(&"Porsche".into()));
    }

    #[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
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

        assert_eq!([1], persons.pk.get(&41));
        assert_eq!([0], persons.pk.get(&3));
        assert!(persons.pk.get(&101).is_empty());

        assert_eq!([0, 1], persons.multi.get(&7));

        let f = Filter(&persons.multi);
        assert_eq!([0, 1], f.eq(&3) | f.eq(&7));

        assert_eq!([0], persons.name.get(&"Jasmin".into()));

        let f = Filter(&persons.name);
        assert_eq!([0, 1], f.eq(&"Jasmin".into()) | f.eq(&"Mario".into()));

        assert_eq!([1, 2], persons.gender.get(&Male));
        assert_eq!([0], persons.gender.get(&Female));
    }

    #[test]
    fn different_idxs() {
        use crate::index::store::Store;
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

        let fname = Filter(&name);
        let fgender = Filter(&gender);

        assert_eq!([1], fgender.eq(&Female) & fname.eq(&"Julia"));

        // = "z" or = 1 and = "a" => (= 1 and "a") or "z"
        assert_eq!(
            [0, 1],
            fname.eq(&"z") | fgender.eq(&Female) & fname.eq(&"Julia")
        );
    }
}
