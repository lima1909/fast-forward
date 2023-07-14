//! `Read-Only Collections` with one index.
//!
use std::{collections::HashMap, hash::Hash, marker::PhantomData, ops::Deref};

use crate::{
    collections::Retriever,
    index::{
        store::{Store, ToStore},
        Indexable,
    },
};

/// [`IList`] is a read only `List` (Vec, Array, ..., default is a Vec) which owned the given items.
/// The list supported one `Index`.
pub struct IList<S, T, L = Vec<T>> {
    store: S,
    items: L,
    _type: PhantomData<T>,
}

impl<S, T, L> IList<S, T, L>
where
    S: Store<Index = usize>,
    L: Indexable<usize, Output = T>,
{
    pub fn new<F, K>(field: F, items: L) -> Self
    where
        F: Fn(&T) -> K,
        S: Store<Key = K, Index = usize>,
        L: ToStore<usize, T>,
    {
        Self {
            store: items.to_store(field),
            items,
            _type: PhantomData,
        }
    }

    pub fn idx(&self) -> Retriever<'_, S, L> {
        Retriever::new(&self.store, &self.items)
    }
}

impl<S, T, L> Deref for IList<S, T, L> {
    type Target = L;

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

/// [`IRefList`] is a read only `List` with a reference (borrowed) to the given items.
/// The list supported one `Index`.
pub struct IRefList<'l, S, T> {
    store: S,
    items: Slice<'l, T>,
}

impl<'l, S, T> IRefList<'l, S, T>
where
    S: Store<Index = usize>,
{
    pub fn new<F, K>(field: F, items: &'l [T]) -> Self
    where
        F: Fn(&T) -> K,
        S: Store<Key = K, Index = usize>,
    {
        Self {
            store: items.to_store(field),
            items: Slice(items),
        }
    }

    pub fn idx(&self) -> Retriever<'_, S, Slice<'l, T>> {
        Retriever::new(&self.store, &self.items)
    }
}

impl<S, T> Deref for IRefList<'_, S, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.items.0
    }
}

/// Wrapper for `slices`.
#[repr(transparent)]
pub struct Slice<'s, T>(pub &'s [T]);

impl<'s, T> Deref for Slice<'s, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'s, T> Indexable<usize> for Slice<'s, T> {
    type Output = T;

    fn item(&self, idx: &usize) -> &Self::Output {
        self.0.item(idx)
    }
}

/// [`IMap`] is a read only `Key-Value-Map` with one index.
pub struct IMap<S, X, T, M = HashMap<X, T>> {
    store: S,
    items: M,
    _idx: PhantomData<X>,
    _type: PhantomData<T>,
}

impl<S, X, T, M> IMap<S, X, T, M>
where
    S: Store<Index = X>,
    M: Indexable<X>,
{
    pub fn new<F, K>(field: F, items: M) -> Self
    where
        F: Fn(&T) -> K,
        S: Store<Key = K, Index = X>,
        X: Eq + Hash + Clone,
        M: ToStore<X, T>,
    {
        Self {
            store: items.to_store(field),
            items,
            _idx: PhantomData,
            _type: PhantomData,
        }
    }

    pub fn idx(&self) -> Retriever<'_, S, M> {
        Retriever::new(&self.store, &self.items)
    }
}

impl<S, X, T, M> Deref for IMap<S, X, T, M> {
    type Target = M;

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use super::*;
    use crate::index::{map::MapIndex, uint::UIntIndex};
    use rstest::{fixture, rstest};

    #[derive(Debug, PartialEq)]
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
    fn ilist_vec(cars: Vec<Car>) {
        let l = IList::<UIntIndex, _>::new(Car::id, cars);

        // deref
        assert_eq!(4, l.len());
        assert_eq!(Car(2, "BMW".into()), l[0]);

        // store
        assert!(l.idx().contains(&2));
        assert!(!l.idx().contains(&2000));

        let mut it = l.idx().get(&2);
        assert_eq!(Some(&Car(2, "BMW".into())), it.next());
        assert_eq!(Some(&Car(2, "VW".into())), it.next());
        assert_eq!(None, it.next());

        let mut it = l.idx().get_many([99, 5]);
        assert_eq!(Some(&Car(99, "Porsche".into())), it.next());
        assert_eq!(Some(&Car(5, "Audi".into())), it.next());
        assert_eq!(None, it.next());

        let mut it = l.idx().filter(|f| {
            assert!(f.contains(&99));

            let idxs = f.eq(&99);
            assert_eq!([3], idxs);

            let mut it = f.items(&99);
            assert_eq!(Some(&Car(99, "Porsche".into())), it.next());
            assert_eq!(None, it.next());

            idxs
        });
        assert_eq!(Some(&Car(99, "Porsche".into())), it.next());
        assert_eq!(None, it.next());

        assert_eq!(2, l.idx().meta().min());
        assert_eq!(99, l.idx().meta().max());
    }

    #[rstest]
    fn ilist_vecdeque(cars: Vec<Car>) {
        let cars = VecDeque::from_iter(cars.into_iter());
        let l = IList::<UIntIndex, _, VecDeque<_>>::new(Car::id, cars);

        // deref
        assert_eq!(4, l.len());
        assert_eq!(Car(2, "BMW".into()), l[0]);
        assert_eq!(&Car(99, "Porsche".into()), l.back().unwrap());

        // store
        assert!(l.idx().contains(&2));
        assert!(!l.idx().contains(&2000));

        let mut it = l.idx().get(&2);
        assert_eq!(Some(&Car(2, "BMW".into())), it.next());
        assert_eq!(Some(&Car(2, "VW".into())), it.next());
        assert_eq!(None, it.next());
    }

    #[test]
    fn ilist_hashmap_str() {
        use std::collections::HashMap;

        let mut m = HashMap::new();
        m.insert("BMW", Car(2, "BMW".into()));
        m.insert("Audi", Car(5, "Audi".into()));
        m.insert("VW", Car(2, "VW".into()));
        m.insert("Porsche", Car(99, "Porsche".into()));

        let l: IMap<UIntIndex<usize, &'static str>, _, Car> = IMap::new(Car::id, m);

        assert_eq!(4, l.len());
        assert_eq!(Car(2, "BMW".into()), l["BMW"]);

        assert!(l.idx().contains(&2));
        assert!(!l.idx().contains(&200));

        let mut it = l.idx().get(&2);
        assert_eq!(Some(&Car(2, "BMW".into())), it.next());
        assert_eq!(Some(&Car(2, "VW".into())), it.next());
        assert_eq!(None, it.next());

        let mut it = l.idx().get_many([99, 5]);
        assert_eq!(Some(&Car(99, "Porsche".into())), it.next());
        assert_eq!(Some(&Car(5, "Audi".into())), it.next());
        assert_eq!(None, it.next());

        let mut it = l.idx().filter(|f| {
            assert!(f.contains(&99));

            let idxs = f.eq(&99);
            assert_eq!(["Porsche"], idxs.as_slice());

            let mut it = f.items(&99);
            assert_eq!(Some(&Car(99, "Porsche".into())), it.next());
            assert_eq!(None, it.next());

            idxs
        });
        assert_eq!(Some(&Car(99, "Porsche".into())), it.next());
        assert_eq!(None, it.next());

        assert_eq!(2, l.idx().meta().min());
        assert_eq!(99, l.idx().meta().max());
    }

    #[test]
    fn ilist_hashmap_usize() {
        use std::collections::HashMap;

        let mut m = HashMap::<usize, Car>::new();
        m.insert(2, Car(2, "BMW".into()));
        m.insert(5, Car(5, "Audi".into()));
        m.insert(3, Car(3, "VW".into()));
        m.insert(99, Car(99, "Porsche".into()));

        let l: IMap<UIntIndex<usize>, _, Car> = IMap::new(Car::id, m);

        assert_eq!(4, l.len());
        assert_eq!(Car(2, "BMW".into()), l[&2]);

        assert!(l.idx().contains(&2));
        assert!(!l.idx().contains(&200));

        let mut it = l.idx().get(&2);
        assert_eq!(Some(&Car(2, "BMW".into())), it.next());
        assert_eq!(None, it.next());

        let mut it = l.idx().get_many([99, 5]);
        assert_eq!(Some(&Car(99, "Porsche".into())), it.next());
        assert_eq!(Some(&Car(5, "Audi".into())), it.next());
        assert_eq!(None, it.next());

        let mut it = l.idx().filter(|f| {
            assert!(f.contains(&99));

            let idxs = f.eq(&99);
            assert_eq!([99], idxs.as_slice());

            let mut it = f.items(&99);
            assert_eq!(Some(&Car(99, "Porsche".into())), it.next());
            assert_eq!(None, it.next());

            idxs
        });
        assert_eq!(Some(&Car(99, "Porsche".into())), it.next());
        assert_eq!(None, it.next());

        assert_eq!(2, l.idx().meta().min());
        assert_eq!(99, l.idx().meta().max());
    }

    #[test]
    fn ilist_btreemap() {
        use std::collections::BTreeMap;

        let mut m = BTreeMap::new();
        m.insert("BMW", Car(2, "BMW".into()));
        m.insert("Audi", Car(5, "Audi".into()));
        m.insert("VW", Car(2, "VW".into()));
        m.insert("Porsche", Car(99, "Porsche".into()));

        let l: IMap<UIntIndex<usize, &'static str>, _, Car, BTreeMap<_, _>> = IMap::new(Car::id, m);

        // deref
        assert_eq!(4, l.len());
        assert_eq!(Car(2, "BMW".into()), l["BMW"]);
        // deref with BTreeMap method (not by HashMap)
        assert_eq!(
            (&"Audi", &Car(5, "Audi".into())),
            l.first_key_value().unwrap()
        );

        assert!(l.idx().contains(&2));
        assert!(!l.idx().contains(&200));

        let mut it = l.idx().get(&2);
        assert_eq!(Some(&Car(2, "BMW".into())), it.next());
        assert_eq!(Some(&Car(2, "VW".into())), it.next());
        assert_eq!(None, it.next());
    }

    #[rstest]
    fn read_only_index_list_from_array(cars: Vec<Car>) {
        let l: IRefList<'_, UIntIndex, _> = IRefList::<'_, UIntIndex, _>::new(Car::id, &cars);

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
            assert!(f.contains(&99));

            let idxs = f.eq(&99);
            assert_eq!([3], idxs);

            let mut it = f.items(&99);
            assert_eq!(Some(&Car(99, "Porsche".into())), it.next());
            assert_eq!(None, it.next());

            idxs
        });
        assert_eq!(Some(&Car(99, "Porsche".into())), it.next());
        assert_eq!(None, it.next());

        // use cars vec after borrow from IRefList
        assert_eq!(4, cars.len());
    }

    struct Cars<'c> {
        ids: IRefList<'c, UIntIndex, Car>,
        names: IRefList<'c, MapIndex, Car>,
    }

    #[rstest]
    fn read_only_double_index_list_from_vec(cars: Vec<Car>) {
        let ids = IRefList::<'_, UIntIndex, _>::new(Car::id, &cars);
        let names = IRefList::<'_, MapIndex, _>::new(|c: &Car| c.1.clone(), &cars);

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
