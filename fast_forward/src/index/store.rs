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
}

/// Returns a list to the indices [`SelectedIndices`] corresponding to the key.
pub trait Filterable {
    type Key;

    /// Get all indices for a given `Key`.
    /// If the `Key` not exist, than this method returns [`SelectedIndices::empty()`]
    fn get(&self, key: &Self::Key) -> SelectedIndices<'_>;

    /// Checks whether the `Key` exists.
    #[inline]
    fn contains(&self, key: &Self::Key) -> bool {
        !self.get(key).is_empty()
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
                let mut c = self.get(&key);
                for k in it {
                    c = c | self.get(&k)
                }
                c
            }
            None => SelectedIndices::empty(),
        }
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

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use super::*;

    impl<'s> Filterable for Vec<&'s str> {
        type Key = &'s str;

        fn get(&self, key: &Self::Key) -> SelectedIndices<'_> {
            let idx = self.binary_search(key).unwrap();
            SelectedIndices::new(idx)
        }
    }

    struct Filter<'f, F>(&'f F);

    impl<'f, F> Deref for Filter<'f, F> {
        type Target = F;

        fn deref(&self) -> &Self::Target {
            self.0
        }
    }

    trait Two<'f> {
        type Key;

        fn two(&'f self, key1: &Self::Key, key2: &Self::Key) -> SelectedIndices<'f>;
    }

    impl<'f, F: Filterable> Two<'f> for Filter<'f, F> {
        type Key = F::Key;

        fn two(&'f self, key1: &Self::Key, key2: &Self::Key) -> SelectedIndices<'f> {
            self.get(key1) | self.get(key2)
        }
    }

    fn extended_filter<'i>(f: &'i Filter<'i, Vec<&'i str>>, key: &'i &str) -> SelectedIndices<'i> {
        f.get(key)
    }

    #[test]
    fn retrieve_filter() {
        let list = vec!["a", "b", "c"];
        let f = Filter(&list);
        assert!(f.contains(&"a"));
        assert_eq!(&[1], &f.get(&"b"));
        assert_eq!(&[0, 1], &f.get_many(["a", "b"]));
        assert_eq!(&[2], &f.get(&"c"));
    }

    #[test]
    fn retrieve_extend_filter() {
        let list = vec!["a", "b", "c"];
        let f = Filter(&list);

        assert_eq!(&[0, 2], &f.two(&"c", &"a"));
        assert_eq!(&[2], &extended_filter(&f, &"c"));
    }
}
