//! read-write collections.
//!
use std::{fmt::Debug, ops::Deref};

use crate::{
    collections::{rw::Editable, Retriever},
    index::store::Store,
};

use super::{base::List, Editor};

/// [`IList`] is a read write indexed `List` which owned the given items.
#[repr(transparent)]
#[derive(Debug)]
pub struct IList<S, I, F>(List<S, I, F>);

impl<S, I, F> IList<S, I, F>
where
    S: Store<Index = usize>,
    F: Fn(&I) -> S::Key,
{
    pub fn new(field: F) -> Self {
        Self(List::new(field))
    }

    pub fn from_vec(field: F, v: Vec<I>) -> Self {
        Self(List::from_vec(field, v))
    }

    pub fn from_iter<It>(field: F, iter: It) -> Self
    where
        It: IntoIterator<Item = I> + ExactSizeIterator,
    {
        Self(List::from_iter(field, iter))
    }

    /// Append a new `Item` to the List.
    pub fn push(&mut self, item: I) -> usize {
        self.0.push(item)
    }

    /// Update the item on the given position.
    pub fn update<U>(&mut self, pos: usize, update: U) -> Option<&I>
    where
        U: FnMut(&mut I),
    {
        self.0.update(pos, update)
    }

    /// The Item in the list will be removed.
    ///
    /// ## Hint:
    /// The remove is a swap_remove ([`std::vec::Vec::swap_remove`])
    pub fn remove(&mut self, pos: usize) -> Option<I> {
        self.0.remove(pos)
    }

    pub fn idx(&self) -> Retriever<'_, S, Vec<I>> {
        self.0.idx()
    }

    pub fn idx_mut(&mut self) -> Editor<'_, I, List<S, I, F>> {
        Editor::new(&mut self.0)
    }
}

impl<S, I, F> Deref for IList<S, I, F> {
    type Target = [I];

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::{IntIndex, MapIndex, UIntIndex};
    use rstest::{fixture, rstest};

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
        let mut removed = Vec::new();
        l.idx_mut().remove_by_key_with_cb(&2, |i| removed.push(i));

        assert_eq!(
            vec![
                Person::new(2, "Mario"),
                Person::new(2, "Peter"),
                Person::new(2, "Jasmin"),
            ],
            removed
        );
        assert_eq!(2, l.len());

        assert_eq!(Some(&Person::new(1, "Inge")), l.idx().get(&1).next());
        l.idx_mut().remove_by_key(&1);
        assert_eq!(None, l.idx().get(&1).next());

        // key not exist
        removed.clear();
        l.idx_mut().remove_by_key_with_cb(&99, |i| removed.push(i));
        assert!(removed.is_empty());
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
        let mut removed = Vec::new();
        l.idx_mut()
            .remove_by_key_with_cb(&"Paul".into(), |i| removed.push(i));

        assert_eq!(
            vec![Person::new(0, "Paul"), Person::new(2, "Paul"),],
            removed
        );
        assert_eq!(3, l.len());

        // key not exist
        removed.clear();
        l.idx_mut()
            .remove_by_key_with_cb(&"Noooo".into(), |i| removed.push(i));
        assert!(removed.is_empty());
    }

    #[test]
    fn ilist_usize() {
        #[derive(Debug, PartialEq)]
        pub struct Person(i32, &'static str);

        #[allow(clippy::useless_conversion)]
        let mut s = IList::<IntIndex, _, _>::from_iter(|p| p.0, vec![Person(-1, "A")].into_iter());
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

        #[allow(clippy::useless_conversion)]
        let mut s = IList::<MapIndex<&'static str, usize>, _, _>::from_iter(
            |p| p.1.clone(),
            vec![Person(-1, "A")].into_iter(),
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
        #[allow(clippy::useless_conversion)]
        let cars = IList::<UIntIndex, _, _>::from_iter(|c| c.0, cars.into_iter());
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

    #[rstest]
    fn update_by_key(cars: Vec<Car>) {
        let mut cars = IList::<UIntIndex, _, _>::from_vec(|c| c.0, cars);
        let mut updated = Vec::new();
        cars.idx_mut().update_by_key_with_cb(
            &2,
            |c| {
                c.1.push_str("_NEW");
            },
            |c| updated.push(c.1.clone()),
        );

        // update many
        assert_eq!(
            vec![String::from("BMW_NEW"), String::from("VW_NEW")],
            updated
        );

        // update one
        cars.idx_mut().update_by_key(&99, |c| {
            c.1.push_str("_NEW");
        });
        assert_eq!(
            Some(&Car(99, "Porsche_NEW".into())),
            cars.idx().get(&99).next()
        );

        // update not found
        updated.clear();
        cars.idx_mut().update_by_key_with_cb(
            &10_000,
            |c| {
                c.1.push_str("_NEW");
            },
            |c| updated.push(c.1.clone()),
        );
        assert!(updated.is_empty());
    }
}
