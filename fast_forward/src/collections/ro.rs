//! `Read-Only-List` with one index.
//!
use std::{
    borrow::Cow,
    ops::{Deref, Index},
};

use crate::{
    collections::Retriever,
    index::{Filterable, Store},
};

// [`ROIndexList`] is a read only list with one index.
//
pub struct ROIndexList<'i, I, S>
where
    [I]: ToOwned,
{
    items: Slice<'i, I>,
    store: S,
}

impl<'i, I, S> ROIndexList<'i, I, S>
where
    [I]: ToOwned,
{
    pub fn borrowed<K, F>(field: F, items: &'i [I]) -> Self
    where
        F: Fn(&I) -> K,
        S: Store<Key = K>,
    {
        Self {
            store: S::from_iter(items.iter().map(field)),
            items: Slice(Cow::Borrowed(items)),
        }
    }

    pub fn owned<K, F>(field: F, items: Vec<I>) -> Self
    where
        F: Fn(&I) -> K,
        S: Store<Key = K>,
    {
        Self {
            store: S::from_iter(items.iter().map(field)),
            items: Slice(Cow::Owned(items.to_owned())),
        }
    }

    pub fn idx(&self) -> Retriever<'_, S, Slice<'_, I>>
    where
        S: Filterable,
        [I]: ToOwned,
    {
        Retriever::new(&self.store, &self.items)
    }
}

impl<'i, I, S> Deref for ROIndexList<'i, I, S>
where
    [I]: ToOwned,
{
    type Target = [I];

    fn deref(&self) -> &Self::Target {
        &self.items.0
    }
}

/// Wrapper for `slices`.
#[repr(transparent)]
pub struct Slice<'s, I>(pub Cow<'s, [I]>)
where
    [I]: ToOwned;

impl<'s, I> Index<usize> for Slice<'s, I>
where
    [I]: ToOwned,
{
    type Output = I;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::{map::MapIndex, uint::UIntIndex};
    use rstest::{fixture, rstest};

    #[derive(Debug, Eq, PartialEq, Clone)]
    pub struct Car(usize, String);

    impl Car {
        fn id(&self) -> usize {
            self.0
        }
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
    fn read_only_index_list_from_vec(cars: Vec<Car>) {
        let l: ROIndexList<'_, _, UIntIndex> = ROIndexList::borrowed(Car::id, &cars);

        // deref
        assert_eq!(4, l.len());
        assert_eq!(Car(2, "BMW".into()), l[0]);

        // store
        let mut it = l.idx().get(&2);
        assert_eq!(Some(&Car(2, "BMW".into())), it.next());
        assert_eq!(Some(&Car(2, "VW".into())), it.next());
        assert_eq!(None, it.next());

        assert!(l.idx().contains(&99));

        let mut it = l.idx().filter(|f| {
            let idxs = f.eq(&99);
            assert_eq!([3], idxs);

            let mut it = f.items(&99);
            assert_eq!(Some(&Car(99, "Porsche".into())), it.next());
            assert_eq!(None, it.next());

            idxs
        });
        assert_eq!(Some(&Car(99, "Porsche".into())), it.next());
        assert_eq!(None, it.next());

        // use cars vec after borrow from ROIndexList
        assert_eq!(4, cars.len());
    }

    #[rstest]
    fn owned_vec_for_ro_list(cars: Vec<Car>) {
        let l: ROIndexList<'_, _, UIntIndex> = ROIndexList::owned(Car::id, cars);

        // deref
        assert_eq!(4, l.len());
        assert_eq!(Car(2, "BMW".into()), l[0]);

        // use cars vec after borrow from ROIndexList
        // not possible: borrow of moved value: `cars`
        // assert_eq!(4, cars.len());
    }

    struct Cars<'c> {
        ids: ROIndexList<'c, Car, UIntIndex>,
        names: ROIndexList<'c, Car, MapIndex>,
    }

    #[rstest]
    fn read_only_double_index_list_from_vec(cars: Vec<Car>) {
        let ids: ROIndexList<'_, _, UIntIndex> = ROIndexList::borrowed(Car::id, &cars);
        let names: ROIndexList<'_, _, MapIndex> =
            ROIndexList::borrowed(|c: &Car| c.1.clone(), &cars);

        let l = Cars { ids, names };

        let mut it = l.ids.idx().get(&2);
        assert_eq!(Some(&Car(2, "BMW".into())), it.next());
        assert_eq!(Some(&Car(2, "VW".into())), it.next());
        assert_eq!(None, it.next());

        let mut it = l.names.idx().get(&"VW".into());
        assert_eq!(Some(&Car(2, "VW".into())), it.next());
        assert_eq!(None, it.next());

        // combine two indices: id and name
        let idxs = l.ids.idx().eq(&2) & l.names.idx().eq(&"VW".into());
        let mut it = idxs.items(&cars);
        assert_eq!(Some(&Car(2, "VW".into())), it.next());
        assert_eq!(None, it.next());
    }
}
