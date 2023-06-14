//! Read-only-list with one index.
use std::ops::{Deref, Index};

use crate::{
    collections::Retriever,
    index::{Filterable, Store},
};

/// [`ROIndexList`] is a read only list with one index.
pub struct ROIndexList<'l, I, S> {
    store: S,
    items: Slice<'l, I>,
}

impl<'l, I, S> ROIndexList<'l, I, S> {
    pub fn new<K, F>(mut store: S, field: F, items: &'l [I]) -> Self
    where
        F: Fn(&I) -> K,
        S: Store<Key = K>,
    {
        for (pos, item) in items.iter().enumerate() {
            store.insert((field)(item), pos);
        }

        Self {
            store,
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

impl<'l, I, S> Deref for ROIndexList<'l, I, S> {
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
    use crate::index::uint::UIntIndex;

    use super::*;

    #[derive(Debug, Eq, PartialEq, Clone)]
    pub struct Car(usize, String);

    #[test]
    fn read_only_index_list_from_vec() {
        let cars = vec![
            Car(2, "BMW".into()),
            Car(5, "Audi".into()),
            Car(2, "VW".into()),
            Car(99, "Porsche".into()),
        ];

        let l = ROIndexList::new(UIntIndex::with_capacity(cars.len()), |c: &Car| c.0, &cars);

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
}
