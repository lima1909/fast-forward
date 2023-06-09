pub mod list;
pub mod one;

use std::ops::Index;

pub use crate::{
    collections::one::OneIndexList,
    index::{self, Filterable, MetaData, SelectedIndices},
};

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

    pub fn eq(&self, key: &F::Key) -> SelectedIndices<'f> {
        self.filter.get(key)
    }

    pub fn eq_many<It>(&self, keys: It) -> SelectedIndices<'f>
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
    pub fn contains(&self, key: F::Key) -> bool {
        !self.filter.eq(&key).is_empty()
    }

    /// Return filter methods from the `Store`.
    pub fn filter<P>(&self, predicate: P) -> index::Iter<'f, L>
    where
        P: Fn(&Filter<'f, F, L>) -> SelectedIndices<'f>,
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
