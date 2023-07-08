//! A `Store` is saving `Indices` for a given `Key`,
//! with the goal, to get the `Indices` as fast as possible.

use std::ops::Index;

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
    fn insert(&mut self, key: Self::Key, idx: Self::Index);

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
    fn update(&mut self, old_key: Self::Key, idx: Self::Index, new_key: Self::Key) {
        self.delete(old_key, &idx);
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
    fn delete(&mut self, key: Self::Key, idx: &Self::Index);

    /// To reduce memory allocations can create an `Index-store` with capacity.
    fn with_capacity(capacity: usize) -> Self;

    /// Create a new `Store` from a given `List` (array, slice, Vec, ...) with a given `Key`.
    /// The `Index-Type` is `usize`.
    fn from_list<I>(it: I) -> Self
    where
        I: IntoIterator<Item = Self::Key>,
        <I as IntoIterator>::IntoIter: ExactSizeIterator,
        Self: Store<Index = usize>,
        Self: Sized,
    {
        Self::from_map(it.into_iter().enumerate().map(|(x, k)| (k, x)))
    }

    /// Create a new `Store` from a given `Map` (`Key-Index-Pair`).
    fn from_map<I>(it: I) -> Self
    where
        I: IntoIterator<Item = (Self::Key, Self::Index)> + ExactSizeIterator,
        Self: Sized,
    {
        let mut store = Self::with_capacity(it.len());
        it.into_iter().for_each(|(k, idx)| store.insert(k, idx));
        store
    }
}

/// Returns a list to the indices [`crate::index::indices::Indices`] corresponding to the key.
pub trait Filterable {
    type Key;
    type Index;

    /// Checks whether the `Key` exists.
    fn contains(&self, key: &Self::Key) -> bool;

    /// Get all indices for a given `Key`.
    /// If the `Key` not exist, than this method returns `empty array`.
    fn get(&self, key: &Self::Key) -> &[Self::Index];

    /// Get all indices for a given `Key`, if the `check` functions returns `true`.
    /// If the `Key` not exist, than this method returns `empty array`.
    fn get_with_check<F>(&self, key: &Self::Key, check: F) -> &[Self::Index]
    where
        F: Fn(&Self::Key) -> bool,
    {
        if check(key) {
            return self.get(key);
        }
        &[]
    }

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
}

/// Meta data from the [`Store`], like min or max value of the `Key`.
pub trait MetaData {
    type Meta<'m>
    where
        Self: 'm;

    /// Return meta data from the `Store`.
    fn meta(&self) -> Self::Meta<'_>;
}

/// `Many` is an `Iterator` for the result from [`Filterable::get_many()`].
pub struct Many<'m, F, K>
where
    F: Filterable,
{
    filter: &'m F,
    keys: K,
    iter: std::slice::Iter<'m, F::Index>,
}

impl<'m, F, K> Many<'m, F, K>
where
    F: Filterable,
    K: Iterator<Item = F::Key> + 'm,
{
    pub fn new(filter: &'m F, mut keys: K) -> Self {
        let iter = match keys.next() {
            Some(k) => filter.get(&k).iter(),
            None => [].iter(),
        };

        Self { filter, keys, iter }
    }

    pub fn items<I>(self, items: &'m I) -> impl Iterator<Item = &'m <I as Index<F::Index>>::Output>
    where
        I: Index<F::Index>,
        <I as Index<F::Index>>::Output: Sized,
        F::Index: Clone,
    {
        self.map(|i| &items[i.clone()])
    }

    pub fn items_vec<I>(self, items: &'m I) -> Vec<&'m <I as Index<F::Index>>::Output>
    where
        I: Index<F::Index>,
        <I as Index<F::Index>>::Output: Sized,
        F::Index: Clone,
    {
        self.map(|i| &items[i.clone()]).collect()
    }
}

impl<'m, F, K> Iterator for Many<'m, F, K>
where
    F: Filterable + 'm,
    K: Iterator<Item = F::Key> + 'm,
    Self: 'm,
{
    type Item = &'m F::Index;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.iter.next();

        #[allow(clippy::nonminimal_bool)]
        if !idx.is_none() {
            return idx;
        }

        loop {
            let key = self.keys.next()?;
            let idx = self.filter.get(&key);
            if !idx.is_empty() {
                self.iter = idx.iter();
                return self.iter.next();
            }
        }
    }
}

/// Create a [`Store`] from a given List or Map and
/// a function for mapping a Struct-Field to an Index.
pub trait ToStore<S, T>
where
    S: Store,
{
    fn to_store<F>(&self, field: F) -> S
    where
        F: FnMut(&T) -> S::Key;
}

impl<S, T, const N: usize> ToStore<S, T> for [T; N]
where
    S: Store<Index = usize>,
{
    fn to_store<F>(&self, field: F) -> S
    where
        F: FnMut(&T) -> <S>::Key,
    {
        S::from_list(self.iter().map(field))
    }
}

impl<'a, S, T> ToStore<S, T> for &'a [T]
where
    S: Store<Index = usize>,
{
    fn to_store<F>(&self, field: F) -> S
    where
        F: FnMut(&T) -> <S>::Key,
    {
        S::from_list(self.iter().map(field))
    }
}

impl<S, T> ToStore<S, T> for Vec<T>
where
    S: Store<Index = usize>,
{
    fn to_store<F>(&self, field: F) -> S
    where
        F: FnMut(&T) -> <S>::Key,
    {
        S::from_list(self.iter().map(field))
    }
}

impl<S, T> ToStore<S, T> for std::collections::VecDeque<T>
where
    S: Store<Index = usize>,
{
    fn to_store<F>(&self, field: F) -> S
    where
        F: FnMut(&T) -> <S>::Key,
    {
        S::from_list(self.iter().map(field))
    }
}

impl<X, S, T> ToStore<S, T> for std::collections::HashMap<X, T>
where
    S: Store<Index = X>,
    X: Clone,
{
    fn to_store<F>(&self, mut field: F) -> S
    where
        F: FnMut(&T) -> <S>::Key,
    {
        S::from_map(self.iter().map(|(idx, item)| (field(item), idx.clone())))
    }
}

impl<X, S, T> ToStore<S, T> for std::collections::BTreeMap<X, T>
where
    S: Store<Index = X>,
    X: Clone,
{
    fn to_store<F>(&self, mut field: F) -> S
    where
        F: FnMut(&T) -> <S>::Key,
    {
        S::from_map(self.iter().map(|(idx, item)| (field(item), idx.clone())))
    }
}

#[cfg(test)]
mod tests {
    use super::{super::filter::Filter, *};
    use crate::index::{
        indices::{Indices, KeyIndices},
        map::MapIndex,
    };
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
        type Index = usize;

        fn get(&self, key: &Self::Key) -> &[usize] {
            match self.idx.get(key) {
                Some(i) => i.as_slice(),
                None => &[],
            }
        }

        fn contains(&self, key: &Self::Key) -> bool {
            self.idx.contains_key(key)
        }
    }

    trait Or<'f> {
        type Key;

        fn or(&'f self, key1: &Self::Key, key2: &Self::Key) -> Indices<'f, usize>;
    }

    impl<'f, F: Filterable<Index = usize>> Or<'f> for Filter<'f, F> {
        type Key = F::Key;

        fn or(&'f self, key1: &Self::Key, key2: &Self::Key) -> Indices<'f, usize> {
            self.eq(key1) | self.eq(key2)
        }
    }

    fn extended_filter<'i>(f: &'i Filter<'i, StrIndex>, key: &'static &str) -> &'i [usize] {
        f.0.get(key)
    }

    #[test]
    fn filter() {
        let list = StrIndex::new();
        let f = Filter(&list);

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
        let f = Filter(&list);

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
        let map = MapIndex::from_list(items.clone());
        assert_eq!(expected, map.get_many(keys).items_vec(&items));
    }
}
