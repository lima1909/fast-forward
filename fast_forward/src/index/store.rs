//! A `Store` is saving `Indices` for a given `Key`,
//! with the goal, to get the `Indices` as fast as possible.

use std::ops::Index;

use super::{indices::EMPTY_INDICES, Indices};

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
    /// If the `Key` not exist, than this method returns [`crate::index::indices::EMPTY_INDICES`]
    fn get(&self, key: &Self::Key) -> &[usize];

    /// Combined all given `keys` with an logical `OR`.
    ///
    /// ## Example:
    ///```text
    /// [2, 5, 6] => get(2) OR get(5) OR get(6)
    /// [2..6] => get(2) OR get(3) OR get(4) OR get(5)
    /// ```
    fn get_many<'k, K>(&'k self, keys: K) -> Many<'k, Self, <K as IntoIterator>::IntoIter>
    where
        K: IntoIterator<Item = Self::Key>,
        K: 'k,
        Self: Sized,
    {
        Many::new(self, keys.into_iter())
    }

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

#[derive(Debug)]
#[repr(transparent)]
pub struct Filter<'f, F>(&'f F);

impl<'f, F> Filter<'f, F>
where
    F: Filterable,
{
    pub const fn new(filter: &'f F) -> Self {
        Self(filter)
    }

    #[inline]
    pub fn eq(&self, key: &F::Key) -> Indices<'f> {
        self.0.get(key).into()
    }

    #[inline]
    pub fn contains(&self, key: &F::Key) -> bool {
        self.0.contains(key)
    }

    // ???
    // #[inline]
    // pub fn items<M, B>(
    //     &self,
    //     key: &F::Key,
    //     map: M,
    // ) -> std::iter::Map<std::slice::Iter<'f, usize>, M>
    // where
    //     M: FnMut(&usize) -> B,
    // {
    //     self.0.get(key).iter().map(map)
    // }
}

pub struct Many<'m, F, K> {
    filter: &'m F,
    keys: K,
    iter: std::slice::Iter<'m, usize>,
}

impl<'m, F, K> Many<'m, F, K>
where
    F: Filterable,
    K: Iterator<Item = F::Key> + 'm,
{
    pub fn new(filter: &'m F, mut keys: K) -> Self {
        let iter = match keys.next() {
            Some(k) => filter.get(&k).iter(),
            None => EMPTY_INDICES.iter(),
        };

        Self { filter, keys, iter }
    }

    pub fn items<I>(self, items: &'m I) -> impl Iterator<Item = &'m <I as Index<usize>>::Output>
    where
        I: Index<usize>,
        <I as Index<usize>>::Output: Sized,
    {
        self.map(|i| &items[*i])
    }

    pub fn items_vec<I>(self, items: &'m I) -> Vec<&'m <I as Index<usize>>::Output>
    where
        I: Index<usize>,
        <I as Index<usize>>::Output: Sized,
    {
        self.map(|i| &items[*i]).collect()
    }
}

impl<'m, F, K> Iterator for Many<'m, F, K>
where
    F: Filterable + 'm,
    K: Iterator<Item = F::Key> + 'm,
    Self: 'm,
{
    type Item = &'m usize;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(i) = self.iter.next() {
            return Some(i);
        }

        loop {
            let key = self.keys.next()?;
            self.iter = self.filter.get(&key).iter();
            if let Some(i) = self.iter.next() {
                return Some(i);
            }
        }
    }
}

/// A `View` is a wrapper for an given [`Store`],
/// that can be only use (read only) for [`Filterable`] operations.
// #[repr(transparent)]
// pub struct View<S>(S);

// impl<S: Store> View<S> {
//     pub fn new<I>(keys: I) -> Self
//     where
//         I: IntoIterator<Item = S::Key> + ExactSizeIterator,
//         Self: Sized,
//     {
//         Self(S::from_iter(keys))
//     }
// }

// impl<S: Store> Filterable for View<S> {
//     type Key = S::Key;

//     #[inline]
//     fn get(&self, key: &Self::Key) -> Indices<'_> {
//         self.0.get(key)
//     }

//     #[inline]
//     fn contains(&self, key: &Self::Key) -> bool {
//         self.0.contains(key)
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::{map::MapIndex, KeyIndices, Store};
    use rstest::rstest;
    use std::collections::HashMap;

    struct StrIndex {
        idx: HashMap<&'static str, KeyIndices>,
    }

    impl StrIndex {
        fn new() -> Self {
            let mut double_a = KeyIndices::new(0);
            double_a.add(3);

            let mut idx = HashMap::new();
            idx.insert("a", double_a);
            idx.insert("b", KeyIndices::new(1));
            idx.insert("c", KeyIndices::new(2));
            idx.insert("s", KeyIndices::new(4));
            Self { idx }
        }
    }

    impl Filterable for StrIndex {
        type Key = &'static str;

        fn get(&self, key: &Self::Key) -> &[usize] {
            match self.idx.get(key) {
                Some(i) => i.as_slice(),
                None => EMPTY_INDICES,
            }
        }
    }

    trait Or<'f> {
        type Key;

        fn or(&'f self, key1: &Self::Key, key2: &Self::Key) -> Indices<'f>;
    }

    impl<'f, F: Filterable> Or<'f> for Filter<'f, F> {
        type Key = F::Key;

        fn or(&'f self, key1: &Self::Key, key2: &Self::Key) -> Indices<'f> {
            self.eq(key1) | self.eq(key2)
        }
    }

    fn extended_filter<'i>(f: &'i Filter<'i, StrIndex>, key: &'static &str) -> &'i [usize] {
        f.0.get(key)
    }

    #[test]
    fn filter() {
        let list = StrIndex::new();
        let f = Filter::new(&list);

        assert!(f.contains(&"a"));
        assert!(!f.contains(&"zz"));

        assert_eq!([1], f.eq(&"b"));
        assert_eq!([0, 1, 3], (f.eq(&"a") | f.eq(&"b")));
        assert_eq!([2], f.eq(&"c"));
        assert_eq!([], f.eq(&"zz"));
    }

    #[test]
    fn extend_filter() {
        let list = StrIndex::new();
        let f = Filter::new(&list);

        assert_eq!([0, 2, 3], f.or(&"c", &"a"));
        assert_eq!([0, 3], f.or(&"zz", &"a"));
        assert_eq!([], f.or(&"zz", &"xx"));
        assert_eq!([2], extended_filter(&f, &"c"));
    }

    #[rstest]
    #[case::empty(vec![], vec![])]
    #[case::one_found(vec!["c"], vec![&"c"])]
    #[case::one_not_found(vec!["-"], vec![])]
    #[case::m_z_a(vec!["m", "z", "a"], vec![&"z", &"a"])]
    #[case::a_m_z(vec![ "a","m", "z"], vec![&"a", &"z"])]
    #[case::z_m_a(vec![ "z","m", "a"], vec![&"z", &"a"])]
    #[case::m_z_a_m(vec!["m", "z", "a", "m"], vec![&"z", &"a"])]
    #[case::m_z_a_m_m(vec!["m", "z", "a", "m", "m"], vec![&"z", &"a"])]
    #[case::double_x(vec!["x"], vec![&"x", &"x"])]
    #[case::a_double_x(vec!["a", "x"], vec![&"a", &"x", &"x"])]
    fn view_str(#[case] keys: Vec<&str>, #[case] expected: Vec<&&str>) {
        let items = vec!["x", "a", "b", "c", "x", "y", "z"];
        let map = MapIndex::from_iter(items.clone().into_iter());
        assert_eq!(expected, map.get_many(keys).items_vec(&items));
    }
}
