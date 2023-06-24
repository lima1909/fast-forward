//! A `Store` is saving `Indices` for a given `Key`,
//! with the goal, to get the `Indices` as fast as possible.
use super::Indices;

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

    /// Create a new `Store` with `Key`-values by given `Iterator`.
    fn from_iter<I>(it: I) -> Self
    where
        I: IntoIterator<Item = Self::Key> + ExactSizeIterator,
        Self: Sized,
    {
        let mut store = Self::with_capacity(it.len());

        for (idx, k) in it.into_iter().enumerate() {
            store.insert(k, idx)
        }

        store
    }
}

/// Returns a list to the indices [`Indices`] corresponding to the key.
pub trait Filterable {
    type Key;

    /// Get all indices for a given `Key`.
    /// If the `Key` not exist, than this method returns [`Indices::empty()`]
    fn get(&self, key: &Self::Key) -> Indices<'_>;

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
    fn get_many<I>(&self, keys: I) -> Indices<'_>
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
            None => Indices::empty(),
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

/// A `View` is a wrapper for an given [`Store`],
/// that can be only use (read only) for [`Filterable`] operations.
#[repr(transparent)]
pub struct View<S>(S);

impl<S: Store> View<S> {
    pub fn new<I>(keys: I) -> Self
    where
        I: IntoIterator<Item = S::Key> + ExactSizeIterator,
        Self: Sized,
    {
        Self(S::from_iter(keys))
    }
}

impl<S: Store> Filterable for View<S> {
    type Key = S::Key;

    #[inline]
    fn get(&self, key: &Self::Key) -> Indices<'_> {
        self.0.get(key)
    }

    #[inline]
    fn get_many<I>(&self, keys: I) -> Indices<'_>
    where
        I: IntoIterator<Item = Self::Key>,
    {
        self.0.get_many(keys)
    }

    #[inline]
    fn contains(&self, key: &Self::Key) -> bool {
        self.0.contains(key)
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use super::*;

    impl<'s> Filterable for Vec<&'s str> {
        type Key = &'s str;

        fn get(&self, key: &Self::Key) -> Indices<'_> {
            if let Ok(idx) = self.binary_search(key) {
                return idx.into();
            }
            Indices::empty()
        }
    }

    struct Filter<'f, F>(&'f F);

    impl<'f, F> Deref for Filter<'f, F> {
        type Target = F;

        fn deref(&self) -> &Self::Target {
            self.0
        }
    }

    trait Or<'f> {
        type Key;

        fn or(&'f self, key1: &Self::Key, key2: &Self::Key) -> Indices<'f>;
    }

    impl<'f, F: Filterable> Or<'f> for Filter<'f, F> {
        type Key = F::Key;

        fn or(&'f self, key1: &Self::Key, key2: &Self::Key) -> Indices<'f> {
            self.get(key1) | self.get(key2)
        }
    }

    fn extended_filter<'i>(f: &'i Filter<'i, Vec<&'i str>>, key: &'i &str) -> Indices<'i> {
        f.get(key)
    }

    #[test]
    fn filter() {
        let list = vec!["a", "b", "c"];
        let f = Filter(&list);
        assert!(f.contains(&"a"));
        assert_eq!(&[1], &f.get(&"b"));
        assert_eq!(&[0, 1], &f.get_many(["a", "b"]));
        assert_eq!(&[2], &f.get(&"c"));
        assert_eq!(&[], &f.get(&"zz"));
    }

    #[test]
    fn extend_filter() {
        let list = vec!["a", "b", "c"];
        let f = Filter(&list);

        assert_eq!(&[0, 2], &f.or(&"c", &"a"));
        assert_eq!(&[0], &f.or(&"zz", &"a"));
        assert_eq!(&[], &f.or(&"zz", &"xx"));
        assert_eq!(&[2], &extended_filter(&f, &"c"));
    }
}
