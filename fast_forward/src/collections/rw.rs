//! read-write collections.
//!
use crate::{
    collections::{base::Retain, Retriever},
    index::{store::Store, Indexable},
};

/// [`IList`] is a read write indexed `List` which owned the given items.
pub struct IList<S, I, F> {
    store: S,
    field: F,
    items: Retain<I>,
}

impl<S, I, F> IList<S, I, F>
where
    S: Store<Index = usize>,
    F: Fn(&I) -> S::Key,
{
    pub fn from_iter<It>(field: F, iter: It) -> Self
    where
        It: IntoIterator<Item = I> + ExactSizeIterator,
    {
        let mut s = Self {
            store: S::with_capacity(iter.len()),
            field,
            items: Retain::with_capacity(iter.len()),
        };

        iter.into_iter().for_each(|item| {
            s.insert(item);
        });

        s
    }

    /// Get the Item on the given position/index in the List.
    /// If the Item was deleted, the return value is `None`
    pub fn get(&self, index: usize) -> Option<&I> {
        self.items.get(index)
    }

    /// Insert a new `Item` to the List.
    pub fn insert(&mut self, item: I) -> usize {
        self.items.insert(item, |item, idx| {
            let key = (self.field)(item);
            self.store.insert(key, idx);
        })
    }

    /// Update the item on the given position.
    pub fn update<U>(&mut self, pos: usize, mut update: U) -> bool
    where
        U: FnMut(&mut I),
    {
        self.items.get_mut(pos).map_or(false, |item| {
            let key = (self.field)(item);
            update(item);
            self.store.update(key, pos, (self.field)(item));
            true
        })
    }

    /// The Item in the list will be marked as deleted.
    pub fn remove(&mut self, pos: usize) -> Option<&I> {
        self.items.drop(pos, |item| {
            let key = (self.field)(item);
            self.store.delete(key, &pos);
        })
    }

    pub fn idx(&self) -> Retriever<'_, S, Retain<I>> {
        Retriever::new(&self.store, &self.items)
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn count(&self) -> usize {
        self.items.count()
    }

    /// Check, is the Item on `pos` (`Index`) deleted.
    pub fn is_removed(&self, pos: usize) -> bool {
        self.items.is_droped(pos)
    }

    // Returns all removed `Indices`.
    pub fn removed_indices(&self) -> &[usize] {
        self.items.droped_indices()
    }

    // Returns all removed `Items`.
    pub fn removed_items(&self) -> impl Iterator<Item = &'_ I> {
        self.removed_indices().iter().map(|i| &self.items[*i])
    }

    pub fn iter(&self) -> impl Iterator<Item = &'_ I> {
        self.items.iter()
    }
}

impl<S, I, F> Indexable<usize> for IList<S, I, F> {
    type Output = I;

    fn item(&self, idx: &usize) -> &Self::Output {
        self.items.item(idx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::{IntIndex, MapIndex, UIntIndex};
    use rstest::{fixture, rstest};

    #[derive(Debug, Eq, PartialEq)]
    pub struct Car(usize, String);

    #[test]
    fn ilist_usize() {
        pub struct Person(i32, &'static str);

        let mut s =
            IList::<IntIndex, Person, _>::from_iter(|p| p.0, vec![Person(-1, "A")].into_iter());
        s.insert(Person(1, "B"));
        assert!(s.idx().contains(&-1));

        // remove
        s.remove(0);
        assert!(!s.idx().contains(&-1));

        // update
        assert!(s.idx().contains(&1));
        // update ID from 1 -> 2 (on Index 1)
        s.update(1, |p| p.0 = 2);

        assert!(!s.idx().contains(&1));
        assert!(s.idx().contains(&2));
    }

    #[test]
    fn item_store_str() {
        pub struct Person(i32, &'static str);

        let mut s = IList::<MapIndex<&'static str, usize>, Person, _>::from_iter(
            |p| p.1.clone(),
            vec![Person(-1, "A")].into_iter(),
        );
        s.insert(Person(1, "B"));
        assert!(s.idx().contains(&"A"));

        // remove
        s.remove(0);
        assert!(!s.idx().contains(&"A"));

        // update
        assert!(s.idx().contains(&"B"));
        // update Name from "B" -> "C" (on Index 1)
        s.update(1, |p| p.1 = "C");

        assert!(!s.idx().contains(&"B"));
        assert!(s.idx().contains(&"C"));
    }

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
        let cars = IList::<UIntIndex, Car, _>::from_iter(|c| c.0, cars.into_iter());
        assert_eq!(&Car(5, "Audi".into()), cars.item(&1));
    }

    #[rstest]
    fn iter_after_remove(cars: Vec<Car>) {
        let mut cars = IList::<UIntIndex, Car, _>::from_iter(|c| c.0, cars.into_iter());
        cars.remove(2);
        cars.remove(0);

        let mut iter = cars.iter();
        assert_eq!(Some(&Car(5, "Audi".into())), iter.next());
        assert_eq!(Some(&Car(99, "Porsche".into())), iter.next());
        assert_eq!(None, iter.next());
    }

    #[rstest]
    fn one_indexed_list_filter_uint(cars: Vec<Car>) {
        let cars = IList::<UIntIndex, Car, _>::from_iter(|c| c.0, cars.into_iter());

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
        let cars = IList::<MapIndex, Car, _>::from_iter(|c| c.1.clone(), cars.into_iter());

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
        let mut cars = IList::<UIntIndex, Car, _>::from_iter(|c| c.0, cars.into_iter());

        // update name, where name is NOT a Index
        let updated = cars.update(0, |c| {
            c.1 = "BMW updated".into();
        });
        assert!(updated);

        assert_eq!(
            vec![&Car(2, "BMW updated".into()), &Car(2, "VW".into())],
            cars.idx().get(&2).collect::<Vec<_>>()
        );

        // update ID, where ID is a Index
        let updated = cars.update(0, |c| {
            c.0 = 5;
        });
        assert!(updated);

        assert_eq!(
            vec![&Car(2, "VW".into())],
            cars.idx().get(&2).collect::<Vec<_>>()
        );
        assert_eq!(
            vec![&Car(5, "BMW updated".into()), &Car(5, "Audi".into())],
            cars.idx().get(&5).collect::<Vec<_>>()
        );

        // update wrong ID
        let updated = cars.update(10_000, |_c| {
            panic!("wrong ID, this trigger is never called")
        });
        assert!(!updated);
    }

    #[rstest]
    fn one_indexed_list_remove(cars: Vec<Car>) {
        let mut cars = IList::<UIntIndex, Car, _>::from_iter(|c| c.0, cars.into_iter());

        // before delete: 2 Cars
        let r = cars.idx().get(&2).collect::<Vec<_>>();
        assert_eq!(vec![&Car(2, "BMW".into()), &Car(2, "VW".into())], r);
        assert_eq!(4, cars.count());

        let deleted_car = cars.remove(0);
        assert_eq!(Some(&Car(2, "BMW".into())), deleted_car);
        assert!(cars.get(0).is_none());

        // after delete: 1 Car
        let r = cars.idx().get(&2).collect::<Vec<_>>();
        assert_eq!(vec![&Car(2, "VW".into())], r);
        assert_eq!(3, cars.count());
        assert_eq!(4, cars.len());
        assert!(!cars.is_empty());
        assert!(cars.is_removed(0));
        assert_eq!(&[0], cars.removed_indices());
        assert_eq!(
            vec![&Car(2, "BMW".into())],
            cars.removed_items().collect::<Vec<_>>()
        );

        // delete a second Car
        let deleted_car = cars.remove(3);
        assert_eq!(Some(&Car(99, "Porsche".into())), deleted_car);
        assert_eq!(2, cars.count());
        assert_eq!(4, cars.len());
        assert!(cars.is_removed(3));
        assert_eq!(&[0, 3], cars.removed_indices());
        assert_eq!(
            vec![&Car(2, "BMW".into()), &Car(99, "Porsche".into())],
            cars.removed_items().collect::<Vec<_>>()
        );
    }

    #[rstest]
    fn delete_wrong_id(cars: Vec<Car>) {
        let mut cars = IList::<UIntIndex, Car, _>::from_iter(|c| c.0, cars.into_iter());
        assert_eq!(None, cars.remove(10_000));
    }
}
