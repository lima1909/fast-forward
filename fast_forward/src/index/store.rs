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
    fn contains(&self, key: &Self::Key) -> bool {
        self.0.contains(key)
    }
}

/// Combined all given `keys` with an logical `OR`.
///
/// ## Example:
///```text
/// [2, 5, 6] => get(2) OR get(5) OR get(6)
/// [2..6] => get(2) OR get(3) OR get(4) OR get(5)
/// ```

pub fn eq_many<F, K>(filter: &F, keys: K) -> Indices<'_>
where
    F: Filterable,
    K: IntoIterator<Item = F::Key>,
{
    Many::new(keys.into_iter(), filter)
        .collect::<Vec<_>>()
        .into()
}

/// `Many` collect many [`Indices`] for a given list of `Keys`.
pub struct Many<'m, K, F> {
    keys: K,
    filter: &'m F,
    indices: Indices<'m>,
    pos: usize,
}

impl<'m, K, F> Many<'m, K, F>
where
    F: Filterable,
    K: Iterator<Item = F::Key>,
{
    pub fn new(mut keys: K, filter: &'m F) -> Self {
        let indices = match keys.next() {
            Some(k) => filter.get(&k),
            None => Indices::empty(),
        };

        Self {
            keys,
            filter,
            indices,
            pos: 0,
        }
    }
}

impl<'m, K, F> Iterator for Many<'m, K, F>
where
    F: Filterable,
    K: Iterator<Item = F::Key>,
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(i) = self.indices.get(self.pos) {
            self.pos += 1;
            return Some(*i);
        }

        self.indices = self.filter.get(&self.keys.next()?);
        while self.indices.is_empty() {
            self.indices = self.filter.get(&self.keys.next()?);
        }

        let idx = self.indices[0];
        self.pos = 1;
        Some(idx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::{map::MapIndex, Store};

    use rstest::rstest;
    use std::ops::Deref;

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
        assert_eq!(&[0, 1], &(f.get(&"a") | f.get(&"b")));
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

    #[rstest]
    #[case::empty(vec![], vec![])]
    #[case::one_found(vec!["c"], vec![3])]
    #[case::one_not_found(vec!["-"], vec![])]
    #[case::m_z_a(vec!["m", "z", "a"], vec![1, 6])]
    #[case::a_m_z(vec![ "a","m", "z"], vec![1, 6])]
    #[case::z_m_a(vec![ "z","m", "a"], vec![1, 6])]
    #[case::m_z_a_m(vec!["m", "z", "a", "m"], vec![1, 6])]
    #[case::m_z_a_m_m(vec!["m", "z", "a", "m", "m"], vec![1, 6])]
    #[case::double_x(vec!["x"], vec![0, 4])]
    #[case::a_double_x(vec!["a", "x"], vec![0, 1, 4])]
    fn view_str(#[case] keys: Vec<&str>, #[case] expected: Vec<usize>) {
        let items = vec!["x", "a", "b", "c", "x", "y", "z"];
        let map = MapIndex::from_iter(items.clone().into_iter());
        assert_eq!(expected, eq_many(&map, keys));
    }
}
