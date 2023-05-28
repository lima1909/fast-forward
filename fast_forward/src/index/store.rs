use crate::index::{Filter, IndexFilter, SelectedIndices};

/// A Store is a mapping from a given `Key` to one or many `Indices`.
pub trait Store: Default {
    type Key;

    /// Insert an `Key` for a given `Index`.
    ///
    /// Before:
    ///     Female | 3,4
    /// `Insert: (Male, 2)`
    /// After:
    ///     Male   | 2
    ///     Female | 3,4
    ///
    /// OR (if the `Key` already exist):
    ///
    /// Before:
    ///     Female | 3,4
    /// `Insert: (Female, 2)`
    /// After:
    ///     Female | 2,3,4
    ///
    fn insert(&mut self, key: Self::Key, idx: usize);

    /// Update means: `Key` changed, but `Index` stays the same
    ///
    /// Before:
    ///     Male   | 1,2,5  
    ///     Female | 3,4
    /// `Update: (Male, 2, Female)`
    /// After:
    ///     Male   | 1,5
    ///     Female | 2,3,4
    ///
    /// otherwise (`Key` has exact one `Index`), then remove complete row (`Key` and `Index`).
    ///
    /// Before:
    ///     Male   | 2
    ///     Female | 3,4
    /// `Update: (Male, 2, Female)`
    /// After:
    ///     Female | 2,3,4
    ///
    /// If the old `Key` not exist, then is it a insert with the new `Key`:
    ///
    /// Before:
    ///     Female | 3,4
    /// `Update: (Male, 2, Female)`
    /// After:
    ///     Female | 2,3,4
    fn update(&mut self, old_key: Self::Key, idx: usize, new_key: Self::Key) {
        self.delete(old_key, idx);
        self.insert(new_key, idx);
    }

    /// Delete means: if an `Key` has more than one `Index`, then remove only this `Index`:
    ///
    /// Before:
    ///     Male   | 1,2,5  
    ///     Female | 3,4
    /// `Delete: Male: 2`
    /// After:
    ///     Male   | 1,5
    ///     Female | 3,4
    ///
    /// otherwise (`Key` has exact one `Index`), then remove complete row (`Key` and `Index`).
    ///
    /// Before:
    ///     Male   | 2
    ///     Female | 3,4
    /// `Delete: Male: 2`
    /// After:
    ///     Female | 3,4
    ///
    /// If the `Key` not exist, then is `delete`ignored:
    ///
    /// Before:
    ///     Female | 3,4
    /// `Delete: Male: 2`
    /// After:
    ///     Female | 3,4
    ///
    fn delete(&mut self, key: Self::Key, idx: usize);

    /// To reduce memory allocations can create an `Index-store` with capacity.
    fn with_capacity(capacity: usize) -> Self;

    type Retriever<'a>
    where
        Self: 'a;

    /// Get instances, to provide Store specific read/select operations.
    fn retrieve<'a, I, L>(&'a self, items: &'a L) -> ItemRetriever<'a, Self::Retriever<'a>, L>
    where
        I: 'a,
        L: IndexFilter<Item = I> + 'a,
        <Self as Store>::Retriever<'a>: Retriever;
}

/// Trait for read/select method from a `Store`.
pub trait Retriever {
    type Key;

    /// Get all indices for a given `Key`.
    fn get(&self, key: &Self::Key) -> SelectedIndices<'_>;

    /// Combined all given `keys` with an logical `OR`.
    ///
    /// ## Example:
    ///```text
    /// get_many([2, 5, 6]) => get(2) OR get(5) OR get(6)
    /// get_many(2..6]) => get(2) OR get(3) OR get(4) OR get(5)
    /// ```
    fn get_many<I>(&self, keys: I) -> SelectedIndices<'_>
    where
        I: IntoIterator<Item = Self::Key>,
    {
        let mut it = keys.into_iter();
        match it.next() {
            Some(key) => {
                let mut c = self.get(&key);
                for k in it {
                    c = c | self.get(&k)
                }
                c
            }
            None => SelectedIndices::empty(),
        }
    }

    /// Checks whether the `Key` exists.
    fn contains(&self, key: &Self::Key) -> bool {
        !self.get(key).is_empty()
    }

    type Filter<'f>
    where
        Self: 'f;

    /// Return filter methods from the `Store`.
    fn filter<'r, P>(&'r self, predicate: P) -> SelectedIndices<'_>
    where
        P: Fn(<Self as Retriever>::Filter<'r>) -> SelectedIndices<'_>;

    type Meta<'m>
    where
        Self: 'm;

    /// Return meta data from the `Store`.
    fn meta(&self) -> Self::Meta<'_>;
}

pub struct ItemRetriever<'a, R, L> {
    retrieve: &'a R,
    items: &'a L,
}

impl<'a, R, L> ItemRetriever<'a, R, L>
where
    R: Retriever,
    L: IndexFilter,
{
    pub fn new(retrieve: &'a R, items: &'a L) -> Self {
        Self { retrieve, items }
    }

    /// Get all items for a given `Key`.
    pub fn get(&self, key: &R::Key) -> Filter<'a, L> {
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
        I: IntoIterator<Item = R::Key>,
    {
        let indices = self.retrieve.get_many(keys);
        self.items.filter(indices)
    }

    /// Checks whether the `Key` exists.
    pub fn contains(&self, key: R::Key) -> bool {
        !self.retrieve.get(&key).is_empty()
    }

    /// Return filter methods from the `Store`.
    pub fn filter<P>(&self, predicate: P) -> Filter<'a, L>
    where
        P: Fn(R::Filter<'a>) -> SelectedIndices<'_>,
    {
        let indices = self.retrieve.filter(predicate);
        self.items.filter(indices)
    }

    /// Return meta data from the `Store`.
    pub fn meta(&self) -> R::Meta<'_> {
        self.retrieve.meta()
    }
}

/// Empty Meta, if the `Retriever` no meta data supported.
pub struct NoMeta;

impl NoMeta {
    pub const fn has_no_meta_data(&self) -> bool {
        true
    }
}

#[repr(transparent)]
pub struct EqFilter<'s, R: Retriever>(pub &'s R);

impl<'s, R: Retriever> EqFilter<'s, R> {
    pub fn eq(&self, key: &R::Key) -> SelectedIndices<'s> {
        self.0.get(key)
    }

    pub fn eq_many<I>(&self, keys: I) -> SelectedIndices<'_>
    where
        I: IntoIterator<Item = R::Key>,
    {
        self.0.get_many(keys)
    }

    pub fn contains(&self, key: &R::Key) -> bool {
        self.0.contains(key)
    }
}
