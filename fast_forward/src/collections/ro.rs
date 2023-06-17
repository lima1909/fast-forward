//! Read-only-list with one index.
use std::ops::{Deref, Index};

use crate::{
    collections::Retriever,
    index::{Filterable, Store},
};

/// [`ROIndexList`] is a read only list with one index.
pub struct ROIndexList<'i, I, S> {
    items: Slice<'i, I>,
    store: S,
}

impl<'i, I, S> ROIndexList<'i, I, S> {
    pub fn new<K, F>(field: F, items: &'i [I]) -> Self
    where
        F: Fn(&I) -> K,
        S: Store<Key = K>,
    {
        Self {
            store: S::from_iter(items.iter().map(field)),
            items: Slice(items),
        }
    }

    pub fn idx(&self) -> Retriever<'_, S, Slice<'_, I>>
    where
        S: Filterable,
    {
        Retriever::new(&self.store, &self.items)
    }
}

impl<'i, I, S, F, K> From<(F, &'i [I])> for ROIndexList<'i, I, S>
where
    F: Fn(&I) -> K,
    S: Store<Key = K>,
{
    fn from(from: (F, &'i [I])) -> Self {
        Self::new(from.0, from.1)
    }
}

impl<'i, I, S> Deref for ROIndexList<'i, I, S> {
    type Target = [I];

    fn deref(&self) -> &Self::Target {
        self.items.0
    }
}

/// Wrapper for `slices`.
#[repr(transparent)]
pub struct Slice<'s, I>(&'s [I]);

impl<'s, I> Index<usize> for Slice<'s, I> {
    type Output = I;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

#[cfg(test)]
mod tests {
    use crate::index::{map::MapIndex, uint::UIntIndex};

    use super::*;

    #[derive(Debug, Eq, PartialEq, Clone)]
    pub struct Car(usize, String);

    impl Car {
        fn id(&self) -> usize {
            self.0
        }
    }

    #[test]
    fn read_only_index_list_from_vec() {
        let cars = vec![
            Car(2, "BMW".into()),
            Car(5, "Audi".into()),
            Car(2, "VW".into()),
            Car(99, "Porsche".into()),
        ];

        let l: ROIndexList<'_, _, UIntIndex> = (Car::id, cars.as_slice()).into();

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
            let _porsche = f.get(3); // no panic
            idxs
        });
        assert_eq!(Some(&Car(99, "Porsche".into())), it.next());
        assert_eq!(None, it.next());
    }

    struct Cars<'c> {
        ids: ROIndexList<'c, Car, UIntIndex>,
        names: ROIndexList<'c, Car, MapIndex>,
    }

    #[test]
    fn read_only_double_index_list_from_vec() {
        let v = vec![
            Car(2, "BMW".into()),
            Car(5, "Audi".into()),
            Car(2, "VW".into()),
            Car(99, "Porsche".into()),
        ];

        let ids: ROIndexList<'_, _, UIntIndex> = (Car::id, v.as_slice()).into();
        let names: ROIndexList<'_, _, MapIndex> = (|c: &Car| c.1.clone(), v.as_slice()).into();

        let cars = Cars { ids, names };

        let mut it = cars.ids.idx().get(&2);
        assert_eq!(Some(&Car(2, "BMW".into())), it.next());
        assert_eq!(Some(&Car(2, "VW".into())), it.next());
        assert_eq!(None, it.next());

        let mut it = cars.names.idx().get(&"VW".into());
        assert_eq!(Some(&Car(2, "VW".into())), it.next());
        assert_eq!(None, it.next());

        // combine two indices: id and name
        let idxs = cars.ids.idx().eq(&2) & cars.names.idx().eq(&"VW".into());
        let mut it = idxs.items(&v);
        assert_eq!(Some(&Car(2, "VW".into())), it.next());
        assert_eq!(None, it.next());
    }
}
