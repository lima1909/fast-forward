pub mod list;
pub mod one;

pub use one::OneIndexList;

use crate::index::{
    store::{self, Filterable},
    Filter, IndexFilter, MetaData, Retriever, SelectedIndices,
};

pub struct ItemRetriever<'a, F, L> {
    retrieve: Retriever<'a, F>,
    items: &'a L,
}

impl<'a, F, L> ItemRetriever<'a, F, L>
where
    F: Filterable,
    L: IndexFilter,
{
    pub fn new(retrieve: Retriever<'a, F>, items: &'a L) -> Self {
        Self { retrieve, items }
    }

    /// Get all items for a given `Key`.
    pub fn get(&self, key: &F::Key) -> Filter<'a, L> {
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
    pub fn get_many<I>(&self, keys: I) -> Filter<'a, L>
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
    pub fn filter<P>(&self, predicate: P) -> Filter<'a, L>
    where
        P: Fn(&store::Filter<'a, F>) -> SelectedIndices<'a>,
    {
        let indices = self.retrieve.filter(predicate);
        self.items.filter(indices)
    }

    pub fn meta(&self) -> F::Meta<'_>
    where
        F: MetaData,
    {
        self.retrieve.meta()
    }
}
