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

/// Trait for read/select method from a `Store`.
// pub trait Retriever {
//     type Key;

//     /// Get all indices for a given `Key`.
//     fn get(&self, key: &Self::Key) -> SelectedIndices<'_>;

//     /// Combined all given `keys` with an logical `OR`.
//     ///
//     /// ## Example:
//     ///```text
//     /// get_many([2, 5, 6]) => get(2) OR get(5) OR get(6)
//     /// get_many(2..6]) => get(2) OR get(3) OR get(4) OR get(5)
//     /// ```
//     fn get_many<I>(&self, keys: I) -> SelectedIndices<'_>
//     where
//         I: IntoIterator<Item = Self::Key>,
//     {
//         let mut it = keys.into_iter();
//         match it.next() {
//             Some(key) => {
//                 let mut c = self.get(&key);
//                 for k in it {
//                     c = c | self.get(&k)
//                 }
//                 c
//             }
//             None => SelectedIndices::empty(),
//         }
//     }

//     /// Checks whether the `Key` exists.
//     fn contains(&self, key: &Self::Key) -> bool {
//         !self.get(key).is_empty()
//     }

//     type Filter<'f>
//     where
//         Self: 'f;

//     /// Return filter methods from the `Store`.
//     fn filter<'r, P>(&'r self, predicate: P) -> SelectedIndices<'_>
//     where
//         P: Fn(<Self as Retriever>::Filter<'r>) -> SelectedIndices<'_>;

//     type Meta<'m>
//     where
//         Self: 'm;

//     /// Return meta data from the `Store`.
//     fn meta(&self) -> Self::Meta<'_>;
// }

// pub struct ItemRetriever<'a, R, L> {
//     retrieve: &'a R,
//     items: &'a L,
// }

// impl<'a, R, L> ItemRetriever<'a, R, L>
// where
//     R: Retriever,
//     L: IndexFilter,
// {
//     pub fn new(retrieve: &'a R, items: &'a L) -> Self {
//         Self { retrieve, items }
//     }

//     /// Get all items for a given `Key`.
//     pub fn get(&self, key: &R::Key) -> Filter<'a, L> {
//         let indices = self.retrieve.get(key);
//         self.items.filter(indices)
//     }

//     /// Combined all given `keys` with an logical `OR`.
//     ///
//     /// ## Example:
//     ///```text
//     /// get_many([2, 5, 6]) => get(2) OR get(5) OR get(6)
//     /// get_many(2..6]) => get(2) OR get(3) OR get(4) OR get(5)
//     /// ```
//     pub fn get_many<I>(&self, keys: I) -> Filter<'a, L>
//     where
//         I: IntoIterator<Item = R::Key>,
//     {
//         let indices = self.retrieve.get_many(keys);
//         self.items.filter(indices)
//     }

//     /// Checks whether the `Key` exists.
//     pub fn contains(&self, key: R::Key) -> bool {
//         !self.retrieve.get(&key).is_empty()
//     }

//     /// Return filter methods from the `Store`.
//     pub fn filter<P>(&self, predicate: P) -> Filter<'a, L>
//     where
//         P: Fn(R::Filter<'a>) -> SelectedIndices<'_>,
//     {
//         let indices = self.retrieve.filter(predicate);
//         self.items.filter(indices)
//     }

//     /// Return meta data from the `Store`.
//     pub fn meta(&self) -> R::Meta<'_> {
//         self.retrieve.meta()
//     }
// }

/// Empty Meta, if the `Retriever` no meta data supported.
// pub struct NoMeta;

// impl NoMeta {
//     pub const fn has_no_meta_data(&self) -> bool {
//         true
//     }
// }

// #[repr(transparent)]
// pub struct EqFilter<'s, R: Retriever> {
//     retriever: &'s R,
// }

// impl<'s, R: Retriever> EqFilter<'s, R> {
//     pub const fn new(retriever: &'s R) -> Self {
//         Self { retriever }
//     }

//     pub fn eq(&self, key: &R::Key) -> SelectedIndices<'s> {
//         self.retriever.get(key)
//     }

//     pub fn eq_many<I>(&self, keys: I) -> SelectedIndices<'_>
//     where
//         I: IntoIterator<Item = R::Key>,
//     {
//         self.retriever.get_many(keys)
//     }
// }

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

    trait Two {
        type Key;

        fn two(&self, key1: &Self::Key, key2: &Self::Key) -> SelectedIndices<'_>;
    }

    impl<'f, F: Filterable> Two for Filter<'f, F> {
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

        // TODO
        // assert_eq!(
        //     SelectedIndices::owned(vec![0, 2]),
        //     r.filter(|f| f.two(&"c", &"a"))
        // );
        assert_eq!(
            SelectedIndices::new(2),
            r.filter(|f| extended_filter(f, &"c"))
        );
    }
}
