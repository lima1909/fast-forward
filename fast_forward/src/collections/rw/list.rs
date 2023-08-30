//! read-write collections.
//!
use std::{fmt::Debug, ops::Deref};

use crate::{
    collections::{rw::base::List, Retriever},
    index::store::Store,
};

/// [`IList`] is a read write indexed `List` which owned the given items.
pub struct IList<S, I, F> {
    store: S,
    field: F,
    items: List<I>,
}

impl<S, I, F> IList<S, I, F>
where
    S: Store<Index = usize>,
    F: Fn(&I) -> S::Key,
{
    pub fn new(field: F) -> Self {
        Self {
            field,
            store: S::with_capacity(0),
            items: List::with_capacity(0),
        }
    }

    pub fn from_vec(field: F, v: Vec<I>) -> Self {
        #[allow(clippy::useless_conversion)]
        // call into_iter is is necessary, because Vec not impl: ExactSizeIterator
        Self::from_iter(field, v.into_iter())
    }

    pub fn from_iter<It>(field: F, iter: It) -> Self
    where
        It: IntoIterator<Item = I> + ExactSizeIterator,
    {
        let mut s = Self {
            field,
            store: S::with_capacity(iter.len()),
            items: List::with_capacity(iter.len()),
        };

        iter.into_iter().for_each(|item| {
            s.push(item);
        });

        s
    }

    /// Append a new `Item` to the List.
    pub fn push(&mut self, item: I) -> usize {
        self.items.push(item, |i, idx| {
            self.store.insert((self.field)(i), idx);
        })
    }

    /// Update the item on the given position.
    pub fn update<U>(&mut self, pos: usize, mut update: U) -> Option<&I>
    where
        U: FnMut(&mut I),
    {
        self.items.get_mut(pos).map(|item| {
            let key = (self.field)(item);
            update(item);
            self.store.update(key, pos, (self.field)(item));
            &*item
        })
    }

    /// The Item in the list will be removed.
    ///
    /// ## Hint:
    /// The remove is a swap_remove ([`std::vec::Vec::swap_remove`])
    pub fn remove(&mut self, pos: usize) -> Option<I> {
        use super::base::RemoveTriggerKind::*;

        self.items.remove(pos, |trigger, i, idx| match trigger {
            Delete => self.store.delete((self.field)(i), &idx),
            Insert => self.store.insert((self.field)(i), idx),
        })
    }

    /// Remove all items by a given `Key`.
    pub fn remove_by_key(&mut self, key: &S::Key) -> Vec<I> {
        let mut removed = Vec::new();

        while let Some(idx) = self.store.get(key).iter().next() {
            if let Some(item) = self.remove(*idx) {
                removed.push(item);
            }
        }
        removed
    }

    pub fn idx(&self) -> Retriever<'_, S, List<I>> {
        Retriever::new(&self.store, &self.items)
    }
}

impl<S, I, F> Deref for IList<S, I, F> {
    type Target = [I];

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl<S, I, F> Debug for IList<S, I, F>
where
    I: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IList").field("items", &self.items).finish()
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;
    use crate::index::{IntIndex, MapIndex, UIntIndex};
    use rstest::{fixture, rstest};

    fn check_key_idx<S, I, F>(l: &mut IList<S, I, F>)
    where
        S: Store<Index = usize>,
        F: Fn(&I) -> S::Key,
    {
        l.items.iter().enumerate().for_each(|(pos, item)| {
            let key = (l.field)(item);
            assert_eq!([pos], l.store.get(&key));
        });
    }

    #[test]
    fn check_key_idx_intindex() {
        let v = vec![
            Person::new(0, "Paul"),
            Person::new(-2, "Mario"),
            Person::new(2, "Jasmin"),
        ];
        check_key_idx(&mut IList::<IntIndex, Person, _>::from_iter(
            |p| p.id,
            v.iter().cloned(),
        ));

        let mut l = IList::<IntIndex, Person, _>::from_iter(|p| p.id, v.iter().cloned());
        l.remove(0);
        check_key_idx(&mut l);

        let mut l = IList::<IntIndex, Person, _>::from_iter(|p| p.id, v.iter().cloned());
        l.remove(1);
        check_key_idx(&mut l);

        let mut l = IList::<IntIndex, Person, _>::from_iter(|p| p.id, v.iter().cloned());
        l.remove(2);
        check_key_idx(&mut l);

        let mut l = IList::<IntIndex, Person, _>::from_iter(|p| p.id, v.iter().cloned());
        l.remove(100);
        check_key_idx(&mut l);

        let mut l = IList::<IntIndex, Person, _>::from_iter(|p| p.id, v.iter().cloned());
        l.remove(0);
        check_key_idx(&mut l);
        l.remove(0);
        check_key_idx(&mut l);
        l.remove(0);
        check_key_idx(&mut l);
        l.remove(0);
        check_key_idx(&mut l);

        let mut l = IList::<IntIndex, Person, _>::from_iter(|p| p.id, v.iter().cloned());
        l.remove(1);
        check_key_idx(&mut l);
        l.remove(1);
        check_key_idx(&mut l);
        l.remove(1);
        check_key_idx(&mut l);
        l.remove(0);
        check_key_idx(&mut l);
        assert_eq!(0, l.len());
    }

    #[test]
    fn check_key_with_many_idx_intindex() {
        let v = vec![
            Person::new(-2, "Paul"),
            Person::new(-2, "Mario"),
            Person::new(2, "Jasmin"),
        ];

        let mut l = IList::<IntIndex, Person, _>::from_iter(|p| p.id, v.iter().cloned());
        l.remove(0);
        check_key_idx(&mut l);

        let mut l = IList::<IntIndex, Person, _>::from_iter(|p| p.id, v.iter().cloned());
        l.remove(1);
        check_key_idx(&mut l);
    }

    #[derive(PartialEq, Debug, Clone)]
    struct Person {
        id: i32,
        name: String,
    }

    impl Person {
        fn new(id: i32, name: &str) -> Self {
            Self {
                id,
                name: name.into(),
            }
        }
    }

    #[test]
    fn check() {
        let mut l = IList::<IntIndex, Person, _>::new(|p| p.id);
        assert_eq!(0, l.push(Person::new(0, "Paul")));
        assert_eq!(1, l.push(Person::new(-2, "Mario")));
        assert_eq!(2, l.push(Person::new(2, "Jasmin")));

        // retrieve GET
        {
            let mut it = l.idx().get(&-2);
            assert_eq!(Some(&Person::new(-2, "Mario")), it.next());
            assert_eq!(None, it.next());
        }
        // deref
        assert_eq!(3, l.len());
        assert_eq!(Some(&Person::new(-2, "Mario")), l.get(1));
        assert_eq!(&Person::new(-2, "Mario"), &l[1]);

        // update name
        assert_eq!(&Person::new(0, "Paul"), &l[0]); // before
        assert_eq!(
            Some(&Person::new(0, "Egon")),
            l.update(0, |p| p.name = "Egon".into())
        );
        assert_eq!(&Person::new(0, "Egon"), &l[0]); // after

        // update id
        assert_eq!(Some(&Person::new(99, "Egon")), l.update(0, |p| p.id = 99));
        assert_eq!(&Person::new(99, "Egon"), &l[0]); // after
        assert_eq!(&Person::new(99, "Egon"), l.idx().get(&99).next().unwrap());

        // update id and name
        assert_eq!(
            Some(&Person::new(100, "Inge")),
            l.update(0, |p| {
                p.id = 100;
                p.name = "Inge".into()
            })
        );
        assert_eq!(&Person::new(100, "Inge"), l.idx().get(&100).next().unwrap());

        // update invalid
        assert_eq!(None, l.update(10_000, |p| p.id = 99));
    }

    #[fixture]
    fn persons() -> Vec<Person> {
        vec![
            Person::new(0, "Paul"),
            Person::new(-2, "Mario"),
            Person::new(2, "Jasmin"),
        ]
    }

    #[rstest]
    fn remove_0(persons: Vec<Person>) {
        let mut l = IList::<IntIndex, _, _>::from_vec(|p| p.id, persons);
        assert_eq!(&Person::new(0, "Paul"), &l[0]);
        assert_eq!(3, l.len());

        assert_eq!(Some(Person::new(0, "Paul")), l.remove(0));

        assert_eq!(&Person::new(2, "Jasmin"), &l[0]);
        assert_eq!(2, l.len());
        assert_eq!(None, l.idx().get(&0).next());
    }

    #[rstest]
    fn remove_1(persons: Vec<Person>) {
        let mut l = IList::<IntIndex, _, _>::from_vec(|p| p.id, persons);
        assert_eq!(&Person::new(-2, "Mario"), &l[1]);
        assert_eq!(3, l.len());

        assert_eq!(Some(Person::new(-2, "Mario")), l.remove(1));

        assert_eq!(&Person::new(2, "Jasmin"), &l[1]);
        assert_eq!(2, l.len());
        assert_eq!(None, l.idx().get(&-2).next());
    }

    #[rstest]
    fn remove_last_2(persons: Vec<Person>) {
        let mut l = IList::<IntIndex, _, _>::from_vec(|p| p.id, persons);
        assert_eq!(&Person::new(2, "Jasmin"), &l[2]);
        assert_eq!(3, l.len());

        assert_eq!(Some(Person::new(2, "Jasmin")), l.remove(2));

        assert_eq!(2, l.len());
        assert_eq!(None, l.idx().get(&2).next());
    }

    #[rstest]
    fn remove_invalid(persons: Vec<Person>) {
        let mut l = IList::<IntIndex, _, _>::from_vec(|p| p.id, persons);
        assert_eq!(None, l.remove(10_000));

        assert_eq!(3, l.len());
    }

    #[test]
    fn remove_empty() {
        let mut l = IList::<IntIndex, Person, _>::from_vec(|p| p.id, vec![]);
        assert_eq!(None, l.remove(0));
    }

    #[test]
    fn remove_by_key_int() {
        let v = vec![
            Person::new(2, "Mario"),
            Person::new(0, "Paul"),
            Person::new(2, "Peter"),
            Person::new(2, "Jasmin"),
            Person::new(1, "Inge"),
        ];

        let mut l = IList::<IntIndex, Person, _>::from_vec(|p| p.id, v);
        assert_eq!(
            vec![
                Person::new(2, "Mario"),
                Person::new(2, "Peter"),
                Person::new(2, "Jasmin"),
            ],
            l.remove_by_key(&2)
        );
        assert_eq!(2, l.len());

        // key not exist
        assert!(l.remove_by_key(&99).is_empty());
    }

    #[test]
    fn remove_by_key_string() {
        let v = vec![
            Person::new(2, "Mario"),
            Person::new(0, "Paul"),
            Person::new(2, "Paul"),
            Person::new(2, "Jasmin"),
            Person::new(1, "Inge"),
        ];

        let mut l = IList::<MapIndex, Person, _>::from_vec(|p| p.name.clone(), v);
        assert_eq!(
            vec![Person::new(0, "Paul"), Person::new(2, "Paul"),],
            l.remove_by_key(&"Paul".into())
        );
        assert_eq!(3, l.len());

        // key not exist
        assert!(l.remove_by_key(&"Noooo".into()).is_empty());
    }

    #[test]
    fn ilist_usize() {
        #[derive(Debug, PartialEq)]
        pub struct Person(i32, &'static str);

        let mut s = IList::<IntIndex, _, _>::from_vec(|p| p.0, vec![Person(-1, "A")]);
        assert_eq!(1, s.push(Person(1, "B")));
        assert!(s.idx().contains(&-1));

        // remove
        s.remove(0);
        assert!(!s.idx().contains(&-1));

        // update
        assert!(s.idx().contains(&1));
        // update ID from 1 -> 2 (on Index 1)
        assert_eq!(&Person(2, "B"), s.update(0, |p| p.0 = 2).unwrap());

        assert!(!s.idx().contains(&1));
        assert!(s.idx().contains(&2));
    }

    #[test]
    fn item_store_str() {
        #[derive(Debug, PartialEq)]
        pub struct Person(i32, &'static str);

        let mut s = IList::<MapIndex<&'static str, usize>, _, _>::from_vec(
            |p| p.1.clone(),
            vec![Person(-1, "A")],
        );
        assert_eq!(1, s.push(Person(1, "B")));
        assert!(s.idx().contains(&"A"));

        // remove
        assert_eq!(Person(-1, "A"), s.remove(0).unwrap());
        assert!(!s.idx().contains(&"A"));

        // update
        assert!(s.idx().contains(&"B"));
        // update Name from "B" -> "C" (on Index 0, after remove: 0)
        assert_eq!(&Person(1, "C"), s.update(0, |p| p.1 = "C").unwrap());

        assert!(!s.idx().contains(&"B"));
        assert!(s.idx().contains(&"C"));
    }

    #[derive(Debug, Eq, PartialEq)]
    pub struct Car(usize, String);

    #[fixture]
    pub fn cars() -> Vec<Car> {
        vec![
            Car(2, "BMW".into()),
            Car(5, "Audi".into()),
            Car(2, "VW".into()),
            Car(99, "Porsche".into()),
        ]
    }

    #[rstest]
    fn item_from_idx(cars: Vec<Car>) {
        let cars = IList::<UIntIndex, _, _>::from_vec(|c| c.0, cars);
        assert_eq!(&Car(5, "Audi".into()), cars.get(1).unwrap());
    }

    #[rstest]
    fn iter_after_remove(cars: Vec<Car>) {
        let mut cars = IList::<UIntIndex, _, _>::from_vec(|c| c.0, cars);
        cars.remove(2);
        cars.remove(0);

        let mut iter = cars.iter();
        assert_eq!(Some(&Car(99, "Porsche".into())), iter.next());
        assert_eq!(Some(&Car(5, "Audi".into())), iter.next());
        assert_eq!(None, iter.next());
    }

    #[rstest]
    fn one_indexed_list_filter_uint(cars: Vec<Car>) {
        let cars = IList::<UIntIndex, _, _>::from_vec(|c| c.0, cars);

        assert!(cars.idx().contains(&2));
        assert_eq!(Some(&Car(2, "VW".into())), cars.get(2));

        let r = cars.idx().get(&2).collect::<Vec<_>>();
        assert_eq!(vec![&Car(2, "BMW".into()), &Car(2, "VW".into())], r);

        let mut it = cars.idx().get(&5);
        assert_eq!(it.next(), Some(&Car(5, "Audi".into())));
        assert_eq!(it.next(), None);

        let mut it = cars.idx().filter(|f| f.eq(&5));
        assert_eq!(it.next(), Some(&Car(5, "Audi".into())));
        assert_eq!(it.next(), None);

        let mut it = cars.idx().get(&1000);
        assert_eq!(it.next(), None);

        assert_eq!(2, cars.idx().meta().min_key());
        assert_eq!(99, cars.idx().meta().max_key());
    }

    #[rstest]
    fn one_indexed_list_filter_map(cars: Vec<Car>) {
        let cars = IList::<MapIndex, _, _>::from_vec(|c| c.1.clone(), cars);

        assert!(cars.idx().contains(&"BMW".into()));

        let r = cars.idx().get(&"VW".into()).collect::<Vec<_>>();
        assert_eq!(vec![&Car(2, "VW".into())], r);

        let mut it = cars
            .idx()
            .filter(|f| f.eq(&"BMW".into()) | f.eq(&"VW".into()));
        assert_eq!(it.next(), Some(&Car(2, "BMW".into())));
        assert_eq!(it.next(), Some(&Car(2, "VW".into())));
        assert_eq!(it.next(), None);

        let mut it = cars.idx().get(&"NotFound".into());
        assert_eq!(it.next(), None);
    }

    #[rstest]
    fn one_indexed_list_update(cars: Vec<Car>) {
        let mut cars = IList::<UIntIndex, _, _>::from_vec(|c| c.0, cars);

        // update name, where name is NOT a Index
        assert_eq!(
            &Car(2, "BMW updated".into()),
            cars.update(0, |c| {
                c.1 = "BMW updated".into();
            })
            .unwrap()
        );

        assert_eq!(
            vec![&Car(2, "BMW updated".into()), &Car(2, "VW".into())],
            cars.idx().get(&2).collect::<Vec<_>>()
        );

        // update ID, where ID is a Index
        assert_eq!(
            &Car(5, "BMW updated".into()),
            cars.update(0, |c| {
                c.0 = 5;
            })
            .unwrap()
        );

        assert_eq!(
            vec![&Car(2, "VW".into())],
            cars.idx().get(&2).collect::<Vec<_>>()
        );
        assert_eq!(
            vec![&Car(5, "BMW updated".into()), &Car(5, "Audi".into())],
            cars.idx().get(&5).collect::<Vec<_>>()
        );

        // update wrong ID
        assert_eq!(
            None,
            cars.update(10_000, |_c| {
                panic!("wrong ID, this trigger is never called")
            })
        );
    }

    #[rstest]
    fn one_indexed_list_remove(cars: Vec<Car>) {
        let mut cars = IList::<UIntIndex, _, _>::from_vec(|c| c.0, cars);

        // before delete: 2 Cars
        let r = cars.idx().get(&2).collect::<Vec<_>>();
        assert_eq!(vec![&Car(2, "BMW".into()), &Car(2, "VW".into())], r);
        assert_eq!(4, cars.len());

        assert_eq!(Some(Car(2, "BMW".into())), cars.remove(0));
        assert_eq!(&Car(99, "Porsche".into()), cars.get(0).unwrap());

        // after delete: 1 Car
        let r = cars.idx().get(&2).collect::<Vec<_>>();
        assert_eq!(vec![&Car(2, "VW".into())], r);
        assert_eq!(3, cars.len());
        assert!(!cars.is_empty());

        // delete a second (last) Car
        assert_eq!(Some(Car(2, "VW".into())), cars.remove(2));
        assert_eq!(2, cars.len());
    }

    #[rstest]
    fn delete_wrong_id(cars: Vec<Car>) {
        let mut cars = IList::<UIntIndex, _, _>::from_vec(|c| c.0, cars);
        assert_eq!(None, cars.remove(10_000));
    }
}
