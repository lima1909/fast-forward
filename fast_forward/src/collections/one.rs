use crate::index::{ItemRetriever, Retriever, Store};

use super::list::List;

pub struct ReadOnlyIndexList<S, K, I, F: Fn(&I) -> K> {
    store: S,
    items: List<I>,
    field: F,
}

impl<S, K, I, F> ReadOnlyIndexList<S, K, I, F>
where
    F: Fn(&I) -> K,
    S: Store<Key = K>,
{
    pub fn from_vec(store: S, f: F, items: Vec<I>) -> Self {
        let mut s = Self {
            store,
            field: f,
            items: List::default(),
        };

        items.into_iter().for_each(|item| {
            s.items.insert(item, |it, idx| {
                s.store.insert((s.field)(it), idx);
            });
        });

        s
    }

    pub fn idx<'a>(&'a self) -> ItemRetriever<'a, S::Retriever<'a>, List<I>>
    where
        <S as Store>::Retriever<'a>: Retriever,
    {
        self.store.retrieve(&self.items)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        collections::one::ReadOnlyIndexList,
        index::{uint::UIntIndex, Store},
    };

    #[derive(Debug, Eq, PartialEq, Clone)]
    struct Car(usize, String);

    #[test]
    fn readonly_indexed_list_filter() {
        let cars = vec![
            Car(2, "BMW".into()),
            Car(5, "Audi".into()),
            Car(2, "VW".into()),
            Car(99, "Porsche".into()),
        ];

        let cars =
            ReadOnlyIndexList::from_vec(UIntIndex::with_capacity(cars.len()), |c: &Car| c.0, cars);

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
}
