use crate::{
    collections::list::{Iter, List},
    index::{ItemRetriever, Retriever, Store},
};

pub struct OneIndexList<S, K, I, F: Fn(&I) -> K> {
    store: S,
    items: List<I>,
    field: F,
}

impl<S, K, I, F> OneIndexList<S, K, I, F>
where
    F: Fn(&I) -> K,
    S: Store<Key = K>,
{
    pub fn from_vec<It>(store: S, f: F, iter: It) -> Self
    where
        It: IntoIterator<Item = I>,
    {
        let mut s = Self {
            store,
            field: f,
            items: List::default(),
        };

        iter.into_iter().for_each(|item| {
            s.insert(item);
        });

        s
    }

    pub fn insert(&mut self, item: I) -> usize {
        self.items.insert(item, |it, idx| {
            self.store.insert((self.field)(it), idx);
        })
    }

    pub fn update<U>(&mut self, pos: usize, update_fn: U) -> bool
    where
        U: Fn(&I) -> I,
    {
        self.items
            .update(pos, update_fn, |old: &I, pos: usize, new: &I| {
                self.store.update((self.field)(old), pos, (self.field)(new));
            })
    }

    pub fn delete(&mut self, pos: usize) -> &I {
        self.items
            .delete(pos, |it, idx| self.store.delete((self.field)(it), idx))
    }

    pub fn idx<'a>(&'a self) -> ItemRetriever<'a, S::Retriever<'a>, List<I>>
    where
        <S as Store>::Retriever<'a>: Retriever,
    {
        self.store.retrieve(&self.items)
    }

    pub fn get(&self, index: usize) -> Option<&I> {
        self.items.get(index)
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

    pub fn is_deleted(&self, pos: usize) -> bool {
        self.items.is_deleted(pos)
    }

    pub const fn iter(&self) -> Iter<'_, I> {
        self.items.iter()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        collections::OneIndexList,
        index::{map::MapIndex, uint::UIntIndex, Store},
    };
    use rstest::{fixture, rstest};

    #[derive(Debug, Eq, PartialEq, Clone)]
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
    fn one_indexed_list_filter_uint(cars: Vec<Car>) {
        let cars =
            OneIndexList::from_vec(UIntIndex::with_capacity(cars.len()), |c: &Car| c.0, cars);

        assert!(cars.idx().contains(2));
        assert!(cars.get(2).is_some());

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

        assert_eq!(2, cars.idx().meta().min());
        assert_eq!(99, cars.idx().meta().max());
    }

    #[rstest]
    fn one_indexed_list_filter_map(cars: Vec<Car>) {
        let cars = OneIndexList::from_vec(
            MapIndex::with_capacity(cars.len()),
            |c: &Car| c.1.clone(),
            cars,
        );

        assert!(cars.idx().contains("BMW".into()));

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
        let mut cars =
            OneIndexList::from_vec(UIntIndex::with_capacity(cars.len()), |c: &Car| c.0, cars);

        let updated = cars.update(0, |c| {
            let mut c_update = c.clone();
            c_update.1 = "BMW updated".into();
            c_update
        });
        assert!(updated);

        assert_eq!(
            vec![&Car(2, "BMW updated".into()), &Car(2, "VW".into())],
            cars.idx().get(&2).collect::<Vec<_>>()
        );
    }

    #[rstest]
    fn one_indexed_list_delete(cars: Vec<Car>) {
        let mut cars =
            OneIndexList::from_vec(UIntIndex::with_capacity(cars.len()), |c: &Car| c.0, cars);

        // before delete: 2 Cars
        let r = cars.idx().get(&2).collect::<Vec<_>>();
        assert_eq!(vec![&Car(2, "BMW".into()), &Car(2, "VW".into())], r);
        assert_eq!(4, cars.count());

        let deleted_car = cars.delete(0);
        assert_eq!(&Car(2, "BMW".into()), deleted_car);
        assert!(cars.get(0).is_none());

        // after delete: 1 Car
        let r = cars.idx().get(&2).collect::<Vec<_>>();
        assert_eq!(vec![&Car(2, "VW".into())], r);
        assert_eq!(3, cars.count());
        assert_eq!(4, cars.len());
        assert!(cars.is_deleted(0));

        // delete a second Car
        let deleted_car = cars.delete(3);
        assert_eq!(&Car(99, "Porsche".into()), deleted_car);
        assert_eq!(2, cars.count());
        assert_eq!(4, cars.len());
        assert!(cars.is_deleted(3));
    }
}
