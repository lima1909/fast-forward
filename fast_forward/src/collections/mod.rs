pub mod list;
pub mod one;

pub use one::OneIndexList;

use crate::index::{
    self,
    store::{Filter, Filterable},
    IndexFilter, MetaData, Retriever, SelectedIndices,
};

pub struct ItemFilter<'a, F, L> {
    filter: Filter<'a, F>,
    items: &'a L,
}

impl<'a, F, L> ItemFilter<'a, F, L>
where
    F: Filterable,
    L: IndexFilter,
{
    const fn new(filter: Filter<'a, F>, items: &'a L) -> Self {
        Self { filter, items }
    }

    pub fn eq(&self, key: &F::Key) -> SelectedIndices<'a> {
        self.filter.eq(key)
    }

    pub fn eq_many<I>(&self, keys: I) -> SelectedIndices<'a>
    where
        I: IntoIterator<Item = F::Key>,
    {
        self.filter.eq_many(keys)
    }

    pub fn get(&self, index: usize) -> &<L as IndexFilter>::Item {
        &self.items[index]
    }
}

pub struct ItemRetriever<'a, F, L> {
    filter: ItemFilter<'a, F, L>,
    retrieve: Retriever<'a, F>,
    items: &'a L,
}

impl<'a, F, L> ItemRetriever<'a, F, L>
where
    F: Filterable,
    L: IndexFilter,
{
    pub const fn new(filter: &'a F, retrieve: Retriever<'a, F>, items: &'a L) -> Self {
        Self {
            filter: ItemFilter::new(Filter(filter), items),
            retrieve,
            items,
        }
    }

    /// Get all items for a given `Key`.
    pub fn get(&self, key: &F::Key) -> index::Iter<'a, L> {
        let indices = self.retrieve.get(key);
        self.items.filter(indices)
    }

    /// Combined all given `keys` with an logical `OR`.
    ///
    /// ## Example:
    ///```text
    /// get_many([2, 5, 6]) => get(2) OR get(5) OR get(6)
    /// get_many(2..6]) => get(2) OR get(3) OR get(4) OR get(5)
    /// ```
    pub fn get_many<I>(&self, keys: I) -> index::Iter<'a, L>
    where
        I: IntoIterator<Item = F::Key>,
    {
        let indices = self.retrieve.get_many(keys);
        self.items.filter(indices)
    }

    /// Checks whether the `Key` exists.
    pub fn contains(&self, key: F::Key) -> bool {
        !self.retrieve.get(&key).is_empty()
    }

    /// Return filter methods from the `Store`.
    pub fn filter<P>(&self, predicate: P) -> index::Iter<'a, L>
    where
        P: Fn(&ItemFilter<'a, F, L>) -> SelectedIndices<'a>,
    {
        let indices = predicate(&self.filter);
        self.items.filter(indices)
    }

    pub fn meta(&self) -> F::Meta<'_>
    where
        F: MetaData,
    {
        self.retrieve.meta()
    }
}
