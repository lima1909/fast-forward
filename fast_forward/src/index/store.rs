use crate::index::SelectedIndices;

/// A Store is a mapping from a given `Key` to one or many `Indices`.
pub trait Store: Filterable {
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

    /// Get instances, to provide Store specific read/select operations.
    fn retrieve(&self) -> Retriever<'_, Self>
    where
        Self: Sized,
    {
        Retriever(Filter(self))
    }
}

/// Meta data from the [`Store`], like min or max value of the `Key`.
pub trait MetaData {
    type Meta<'m>
    where
        Self: 'm;

    /// Return meta data from the `Store`.
    fn meta(&self) -> Self::Meta<'_>;
}

/// Returns a list to the indices [`SelectedIndices`] corresponding to the key.
pub trait Filterable {
    type Key;

    /// Get all indices for a given `Key`.
    /// If the `Key` not exist, than this method returns [`SelectedIndices::empty()`]
    fn indices(&self, key: &Self::Key) -> SelectedIndices<'_>;

    /// Checks whether the `Key` exists.
    #[inline]
    fn contains(&self, key: &Self::Key) -> bool {
        !self.indices(key).is_empty()
    }

    /// Combined all given `keys` with an logical `OR`.
    ///
    /// ## Example:
    ///```text
    /// get_many([2, 5, 6]) => get(2) OR get(5) OR get(6)
    /// get_many(2..6]) => get(2) OR get(3) OR get(4) OR get(5)
    /// ```
    #[inline]
    fn get_many<I>(&self, keys: I) -> SelectedIndices<'_>
    where
        I: IntoIterator<Item = Self::Key>,
    {
        let mut it = keys.into_iter();
        match it.next() {
            Some(key) => {
                let mut c = self.indices(&key);
                for k in it {
                    c = c | self.indices(&k)
                }
                c
            }
            None => SelectedIndices::empty(),
        }
    }
}

#[repr(transparent)]
pub struct Filter<'f, F>(&'f F);

impl<'f, F> Filter<'f, F>
where
    F: Filterable,
{
    pub fn eq(&self, key: &F::Key) -> SelectedIndices<'f> {
        self.0.indices(key)
    }

    pub fn eq_many<I>(&self, keys: I) -> SelectedIndices<'f>
    where
        I: IntoIterator<Item = F::Key>,
    {
        self.0.get_many(keys)
    }
}

/// [`Retriever`] is the entry point for read methods for the [`Store`].
#[repr(transparent)]
pub struct Retriever<'f, F>(Filter<'f, F>);

impl<'f, F> Retriever<'f, F>
where
    F: Filterable,
{
    /// Get all items for a given `Key`.
    pub fn get(&self, key: &F::Key) -> SelectedIndices<'f> {
        self.0.eq(key)
    }

    /// Combined all given `keys` with an logical `OR`.
    ///
    /// ## Example:
    ///```text
    /// get_many([2, 5, 6]) => get(2) OR get(5) OR get(6)
    /// get_many(2..6]) => get(2) OR get(3) OR get(4) OR get(5)
    /// ```
    pub fn get_many<I>(&self, keys: I) -> SelectedIndices<'f>
    where
        I: IntoIterator<Item = F::Key>,
    {
        self.0.eq_many(keys)
    }

    /// Checks whether the `Key` exists.
    pub fn contains(&self, key: &F::Key) -> bool {
        self.0 .0.contains(key)
    }

    /// Return filter methods from the `Store`.
    pub fn filter<P>(&self, predicate: P) -> SelectedIndices<'f>
    where
        P: Fn(&Filter<'f, F>) -> SelectedIndices<'f>,
    {
        predicate(&self.0)
    }

    pub fn meta(&self) -> F::Meta<'_>
    where
        F: MetaData,
    {
        self.0 .0.meta()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl<'s> Filterable for Vec<&'s str> {
        type Key = &'s str;

        fn indices(&self, key: &Self::Key) -> SelectedIndices<'_> {
            let idx = self.binary_search(key).unwrap();
            SelectedIndices::new(idx)
        }
    }

    trait Two<'f> {
        type Key;

        fn two(&self, key1: &Self::Key, key2: &Self::Key) -> SelectedIndices<'f>;
    }

    impl<'f, F: Filterable> Two<'f> for Filter<'f, F> {
        type Key = F::Key;

        fn two(&self, key1: &Self::Key, key2: &Self::Key) -> SelectedIndices<'f> {
            self.eq(key1) | self.eq(key2)
        }
    }

    fn extended_filter<'i>(f: &Filter<'i, Vec<&'i str>>, key: &'i &str) -> SelectedIndices<'i> {
        f.eq(key)
    }

    #[test]
    fn retrieve_filter() {
        let list = vec!["a", "b", "c"];
        let r = Retriever(Filter(&list));
        assert!(r.contains(&"a"));
        assert_eq!(SelectedIndices::new(1), r.get(&"b"));
        assert_eq!(SelectedIndices::owned(vec![0, 1]), r.get_many(["a", "b"]));
        assert_eq!(SelectedIndices::new(2), r.filter(|f| f.eq(&"c")));
    }

    #[test]
    fn retrieve_ignore_filter_eather_func() {
        let list = vec!["a", "b", "c"];
        let r = Retriever(Filter(&list));

        assert_eq!(
            SelectedIndices::new(2),
            r.filter(|_f| SelectedIndices::new(2))
        );
    }

    #[test]
    fn retrieve_extend_filter() {
        let list = vec!["a", "b", "c"];
        let r = Retriever(Filter(&list));

        assert_eq!(
            SelectedIndices::owned(vec![0, 2]),
            r.filter(|f| f.two(&"c", &"a"))
        );
        assert_eq!(
            SelectedIndices::new(2),
            r.filter(|f| extended_filter(f, &"c"))
        );
    }
}
