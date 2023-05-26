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
        index::{uint::UIntIndex, Store},
    };

    #[derive(Debug, Eq, PartialEq, Clone)]
    struct Car(usize, String);

    #[test]
    fn one_indexed_list_filter() {
        let cars = vec![
            Car(2, "BMW".into()),
            Car(5, "Audi".into()),
            Car(2, "VW".into()),
            Car(99, "Porsche".into()),
        ];

        let cars =
            OneIndexList::from_vec(UIntIndex::with_capacity(cars.len()), |c: &Car| c.0, cars);

        assert!(cars.idx().contains(2));

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

    #[test]
    fn one_indexed_list_update() {
        let cars = vec![
            Car(2, "BMW".into()),
            Car(5, "Audi".into()),
            Car(2, "VW".into()),
            Car(99, "Porsche".into()),
        ];

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

    #[test]
    fn one_indexed_list_delete() {
        let cars = vec![
            Car(2, "BMW".into()),
            Car(5, "Audi".into()),
            Car(2, "VW".into()),
            Car(99, "Porsche".into()),
        ];

        let mut cars =
            OneIndexList::from_vec(UIntIndex::with_capacity(cars.len()), |c: &Car| c.0, cars);

        // before delete: 2 Cars
        let r = cars.idx().get(&2).collect::<Vec<_>>();
        assert_eq!(vec![&Car(2, "BMW".into()), &Car(2, "VW".into())], r);
        assert_eq!(4, cars.count());

        let deleted_car = cars.delete(0);
        assert_eq!(&Car(2, "BMW".into()), deleted_car);

        // after delete: 1 Car
        let r = cars.idx().get(&2).collect::<Vec<_>>();
        assert_eq!(vec![&Car(2, "VW".into())], r);
        assert_eq!(3, cars.count());
        // assert!(cars.is_deleted(1));
    }
}
