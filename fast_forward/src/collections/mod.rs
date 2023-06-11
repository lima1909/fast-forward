pub mod list;
pub mod one;

use std::ops::{Deref, Index};

pub use crate::{
    collections::one::OneIndexList,
    index::{self, Filterable, Indices, MetaData, Store},
};

#[repr(transparent)]
pub struct Slice<'s, I>(&'s [I]);

impl<'s, I> Index<usize> for Slice<'s, I> {
    type Output = I;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

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

pub struct Filter<'f, F, I> {
    filter: &'f F,
    items: &'f I,
}

impl<'f, F, I> Filter<'f, F, I>
where
    F: Filterable,
{
    const fn new(filter: &'f F, items: &'f I) -> Self {
        Self { filter, items }
    }

    pub fn eq(&self, key: &F::Key) -> Indices<'f> {
        self.filter.get(key)
    }

    pub fn eq_many<It>(&self, keys: It) -> Indices<'f>
    where
        It: IntoIterator<Item = F::Key>,
    {
        self.filter.get_many(keys)
    }

    pub fn get(&self, i: usize) -> &<I as Index<usize>>::Output
    where
        I: Index<usize>,
    {
        &self.items[i]
    }
}

pub struct Retriever<'f, F, L> {
    filter: Filter<'f, F, L>,
    items: &'f L,
}

impl<'f, F, L> Retriever<'f, F, L>
where
    F: Filterable,
{
    pub const fn new(filter: &'f F, items: &'f L) -> Self {
        Self {
            filter: Filter::new(filter, items),
            items,
        }
    }

    /// Get all items for a given `Key`.
    pub fn get(&self, key: &F::Key) -> index::Iter<'f, L>
    where
        L: Index<usize>,
    {
        self.filter.eq(key).items(self.items)
    }

    /// Combined all given `keys` with an logical `OR`.
    ///
    /// ## Example:
    ///```text
    /// get_many([2, 5, 6]) => get(2) OR get(5) OR get(6)
    /// get_many(2..6]) => get(2) OR get(3) OR get(4) OR get(5)
    /// ```
    pub fn get_many<I>(&self, keys: I) -> index::Iter<'f, L>
    where
        I: IntoIterator<Item = F::Key>,
        L: Index<usize>,
    {
        self.filter.eq_many(keys).items(self.items)
    }

    /// Checks whether the `Key` exists.
    pub fn contains(&self, key: &F::Key) -> bool {
        !self.filter.eq(key).is_empty()
    }

    /// Return filter methods from the `Store`.
    pub fn filter<P>(&self, predicate: P) -> index::Iter<'f, L>
    where
        P: Fn(&Filter<'f, F, L>) -> Indices<'f>,
        L: Index<usize>,
    {
        predicate(&self.filter).items(self.items)
    }

    pub fn meta(&self) -> F::Meta<'_>
    where
        F: MetaData,
    {
        self.filter.filter.meta()
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
