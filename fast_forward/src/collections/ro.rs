//! `Read-Only-List` with one index.
//!
use std::{
    borrow::Cow,
    collections::HashMap,
    hash::Hash,
    marker::PhantomData,
    ops::{Deref, Index},
};

use crate::{collections::Retriever, index::store::Store};

// [`IList`] is a read only `List` with one index.
pub struct IList<S, T, L = Vec<T>>
where
    L: Index<usize>,
{
    store: S,
    items: L,
    _type: PhantomData<T>,
}

impl<S, T, L> IList<S, T, L>
where
    L: Index<usize>,
    S: Store<Index = usize>,
{
    pub fn new<F, K, I>(field: F, items: I) -> Self
    where
        F: Fn(&T) -> K,
        S: Store<Key = K, Index = usize>,
        I: IntoIterator<Item = T>,
        L: FromIterator<T>,
    {
        let v = Vec::from_iter(items);

        Self {
            store: S::from_list(v.iter().map(field)),
            items: L::from_iter(v),
            _type: PhantomData,
        }
    }

    pub fn idx(&self) -> Retriever<'_, S, L> {
        Retriever::new(&self.store, &self.items)
    }
}

impl<S, T, L> Deref for IList<S, T, L>
where
    L: Deref<Target = [T]> + Index<usize>,
{
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl<S, T, L> AsRef<[T]> for IList<S, T, L>
where
    L: AsRef<[T]> + Index<usize>,
{
    fn as_ref(&self) -> &[T] {
        self.items.as_ref()
    }
}

// [`IMap`] is a read only `Key-Value-Map` with one index.
pub struct IMap<S, X, T, M = HashMap<X, T>>
where
    M: Index<X>,
{
    store: S,
    items: M,
    _idx: PhantomData<X>,
    _type: PhantomData<T>,
}

impl<S, X, T, M> IMap<S, X, T, M>
where
    M: Index<X>,
    S: Store<Index = X>,
{
    pub fn new<F, K, I>(field: F, items: I) -> Self
    where
        F: Fn(&T) -> K,
        S: Store<Key = K, Index = X>,
        I: IntoIterator<Item = (X, T)>,
        M: FromIterator<(X, T)>,
        X: Eq + Hash + Clone,
    {
        let items: HashMap<X, T> = HashMap::from_iter(items.into_iter());
        Self {
            store: S::from_map(items.iter().map(|(x, v)| (field(v), x.clone()))),
            items: M::from_iter(items.into_iter()),
            _idx: PhantomData,
            _type: PhantomData,
        }
    }

    pub fn idx(&self) -> Retriever<'_, S, M> {
        Retriever::new(&self.store, &self.items)
    }
}

impl<S, X, T, M> Deref for IMap<S, X, T, M>
where
    M: Index<X>,
    S: Store<Index = X>,
{
    type Target = M;

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

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
    S: Store,
    S::Index: Clone,
{
    pub fn borrowed<K, F>(field: F, items: &'i [I]) -> Self
    where
        F: Fn(&I) -> K,
        S: Store<Key = K, Index = usize>,
    {
        Self {
            store: S::from_list(items.iter().map(field)),
            items: Slice(Cow::Borrowed(items)),
        }
    }

    pub fn owned<K, F>(field: F, items: Vec<I>) -> Self
    where
        F: Fn(&I) -> K,
        S: Store<Key = K, Index = usize>,
    {
        Self {
            store: S::from_list(items.iter().map(field)),
            items: Slice(Cow::Owned(items.to_owned())),
        }
    }

    pub fn idx(&self) -> Retriever<'_, S, Slice<'_, I>>
    where
        S: Store,
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
    fn ilist_vec(cars: Vec<Car>) {
        let l: IList<UIntIndex, _> = IList::new(Car::id, cars);

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

    #[test]
    fn ilist_hashmap() {
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

    // -----------------------------------------------------
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
