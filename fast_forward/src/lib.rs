//! **Fast-Forward** is a library for filtering items in a (large) list, _faster_ than an `Iterator` ([`std::iter::Iterator::filter`]).
//! It is not a replacement of the `Iterator`, rather an addition.
//!
//! This _faster_ is achieved  by using `Indices`. This means, it does not have to touch and compare every item in the list.
//!
//! An `Index` has two parts, a `Key` (item to searching for) and a `Position` (the index) in the list.
//!
//! ## Example:
//!
//! ```text
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
            _items_: $crate::collections::list::List<$item>,
        }

        ///
        impl $fast {

            /// Insert the given item.
            ///
            #[allow(dead_code)]
            fn insert(&mut self, item: $item) -> usize {
                use $crate::index::store::Store;

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
                use $crate::index::store::Store;

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
            fn delete(&mut self, pos: usize) -> Option<&$item> {
                use $crate::index::store::Store;

                self._items_.delete(pos, |it: &$item, pos: &usize| {
                    $(
                        self.$store.delete(
                                    it.$item_field$(.$item_field_func())?,
                                    pos
                                    );
                    )+
                })
            }

            #[allow(dead_code)]
            fn iter(&self) -> $crate::collections::list::Iter<'_, $item> {
                self._items_.iter()
            }

            $(
                /// Create and get a Filter for the Store
                #[allow(dead_code)]
                fn $store(&self) -> $crate::collections::Retriever<'_, $store_type, $crate::collections::list::List<$item>> {
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

    #[derive(Debug, Eq, PartialEq, Clone)]
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
        cars.update(cars.id.get(&99)[0], |c: &Car| Car(c.0 + 1, c.1.clone()));
        let r = cars.id().get(&100).collect::<Vec<_>>();
        assert_eq!(vec![&Car(100, "Porsche".into())], r);

        // remove one Car
        assert!(cars.id().get(&100).next().is_some());
        cars.delete(cars.id.get(&100)[0]);
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
        assert_eq!(2, cars.id.min());
        assert_eq!(99, cars.id.max());
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

    #[derive(Clone, Copy, Default)]
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
