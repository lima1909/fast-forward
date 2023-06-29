//! An `Index` is the position in a list for an giben `Key`.
//!
//! There are three kind of indices:
//! - [`KeyIndices`]:, is using internal in the [`crate::index::Store`] implementation.
//! - [`Indices`]: are a read only reference of indices.
//! - [`CmpIndices`]: list of indices, which you can use in [`std::ops::BitOr`] and [`std::ops::BitAnd`] operations.
//!
use std::{
    borrow::Cow,
    ops::{BitAnd, BitOr, Deref, Index},
};

/// The possibiliy to get `Indices` by a given `Key`.
pub trait Filterable {
    type Key;

    /// Get all indices for a given `Key`.
    /// If the `Key` not exist, than this method returns [`Indices::empty()`]
    fn get<'a>(&'a self, key: &Self::Key) -> Indices<'a>;

    /// Combined all given `Keys` with an logical `OR`.
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

///
/// Is using from the [`crate::index::Store`] to save the `Indices` for a given `Key`.
///
#[repr(transparent)]
pub struct KeyIndices(Vec<usize>);

impl KeyIndices {
    /// Create a new Indices collection with the initial Index.
    #[inline]
    pub fn new(idx: usize) -> Self {
        Self(vec![idx])
    }

    /// Add a new Index to a sorted collection.
    #[inline]
    pub fn add(&mut self, idx: usize) {
        if let Err(pos) = self.0.binary_search(&idx) {
            self.0.insert(pos, idx);
        }
    }

    /// Remove one Index and return left free Indices.
    #[inline]
    pub fn remove(&mut self, idx: usize) -> &Self {
        self.0.retain(|v| v != &idx);
        self
    }
}

impl Deref for KeyIndices {
    type Target = [usize];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

///
/// Is a wrapper for selected `Indices`, e.g. by using the [`Filterable`] trait, the `get` method.
/// You can create an instance with: [`Indices::empty()`] or `KeyIndices::into()` (ordered list of indices).
///
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Indices<'i>(&'i [usize]);

impl<'i> Indices<'i> {
    #[inline]
    pub const fn empty() -> Self {
        Self(&[])
    }
}

impl<'i> IntoIterator for Indices<'i> {
    type Item = &'i usize;
    type IntoIter = std::slice::Iter<'i, usize>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Deref for Indices<'_> {
    type Target = [usize];

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl AsRef<[usize]> for Indices<'_> {
    fn as_ref(&self) -> &[usize] {
        self.0
    }
}

impl<'i> From<&'i KeyIndices> for Indices<'i> {
    fn from(i: &'i KeyIndices) -> Self {
        Indices(&i.0)
    }
}

///
/// Indices for using BitOr and BitAnd operations.
///
#[derive(Debug, PartialEq)]
#[repr(transparent)]
pub struct CmpIndices<'i>(Cow<'i, [usize]>);

impl<'i> From<Indices<'i>> for CmpIndices<'i> {
    fn from(i: Indices<'i>) -> Self {
        Self(Cow::Borrowed(i.0))
    }
}

impl From<KeyIndices> for CmpIndices<'_> {
    fn from(i: KeyIndices) -> Self {
        Self(Cow::Owned(i.0))
    }
}

impl From<Vec<usize>> for CmpIndices<'_> {
    fn from(mut v: Vec<usize>) -> Self {
        // !!! must sorted before can use BitOr or BitAnd !!!
        v.sort();
        Self(Cow::Owned(v))
    }
}

impl Deref for CmpIndices<'_> {
    type Target = [usize];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<[usize]> for CmpIndices<'_> {
    fn as_ref(&self) -> &[usize] {
        &self.0
    }
}

impl BitOr for CmpIndices<'_> {
    type Output = Self;

    fn bitor(self, other: Self) -> Self::Output {
        CmpIndices(super::union(self.0, other.0))
    }
}

impl BitAnd for CmpIndices<'_> {
    type Output = Self;

    fn bitand(self, other: Self) -> Self::Output {
        CmpIndices(super::intersection(self.0, other.0))
    }
}

///
/// Iterator for iterate over many Indices.
///
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
            Some(k) => filter.get(&k).into_iter(),
            None => [].iter(),
        };

        Self { filter, keys, iter }
    }

    pub fn items<I>(self, items: &'m I) -> Items<'_, Many<'_, F, K>, I>
    where
        I: Index<usize>,
        <I as Index<usize>>::Output: Sized,
    {
        Items { iter: self, items }
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
            self.iter = self.filter.get(&key).into_iter();
            if let Some(i) = self.iter.next() {
                return Some(i);
            }
        }
    }
}

pub struct Items<'i, It, I: Index<usize>> {
    iter: It,
    items: &'i I,
}

impl<'i, It, I: Index<usize>> Iterator for Items<'i, It, I>
where
    It: Iterator<Item = &'i usize>,
    <I as Index<usize>>::Output: Sized,
{
    type Item = &'i <I as Index<usize>>::Output;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|i| &self.items[*i])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::collections::HashMap;

    impl<'i> Indices<'i> {
        const fn borrowed(s: &'i [usize]) -> Self {
            Self(s)
        }
    }

    impl<'i> CmpIndices<'i> {
        const fn borrowed(s: &'i [usize]) -> Self {
            Self(Cow::Borrowed(s))
        }

        const fn owned(v: Vec<usize>) -> Self {
            Self(Cow::Owned(v))
        }
    }

    #[allow(non_upper_case_globals)]
    const values: [&str; 5] = ["a", "b", "c", "a", "s"];

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

        fn get<'a>(&'a self, key: &Self::Key) -> Indices<'a> {
            match self.idx.get(key) {
                Some(i) => i.into(),
                None => Indices::empty(),
            }
        }
    }

    struct Filter<'f, F>(&'f F);

    impl<'f, F> Filter<'f, F>
    where
        F: Filterable,
    {
        fn eq(&self, key: &F::Key) -> CmpIndices {
            self.0.get(key).into()
        }
    }

    trait Or<'f> {
        type Key;

        fn or(&'f self, key1: &Self::Key, key2: &Self::Key) -> CmpIndices<'f>;
    }

    impl<'f, F: Filterable> Or<'f> for Filter<'f, F> {
        type Key = F::Key;

        fn or(&'f self, key1: &Self::Key, key2: &Self::Key) -> CmpIndices<'f> {
            self.eq(key1) | self.eq(key2)
        }
    }

    fn extended_filter<'i>(f: &'i Filter<'i, StrIndex>, key: &'static &str) -> Indices<'i> {
        f.0.get(key)
    }

    mod key_indices {
        use super::*;

        impl CmpIndices<'_> {
            fn new(idx: KeyIndices) -> Self {
                idx.into()
            }
        }

        #[test]
        fn unique() {
            assert_eq!([0], KeyIndices::new(0).as_ref());
        }

        #[test]
        fn multi() {
            let mut m = KeyIndices::new(2);
            assert_eq!([2], m.as_ref());

            m.add(1);
            assert_eq!([1, 2], m.as_ref());
        }

        #[test]
        fn multi_duplicate() {
            let mut m = KeyIndices::new(1);
            assert_eq!([1], m.as_ref());

            // ignore add: 1, 1 exists already
            m.add(1);
            assert_eq!([1], m.as_ref());
        }

        #[test]
        fn multi_ordered() {
            let mut m = KeyIndices::new(5);
            assert_eq!([5], m.as_ref());

            m.add(3);
            m.add(1);
            m.add(4);

            assert_eq!([1, 3, 4, 5], m.as_ref());
        }

        #[test]
        fn container_multi() {
            let mut lhs = KeyIndices::new(5);
            lhs.add(3);
            lhs.add(2);
            lhs.add(4);

            let mut rhs = KeyIndices::new(5);
            rhs.add(2);
            rhs.add(9);

            assert_eq!(
                [2, 3, 4, 5, 9],
                (CmpIndices::new(lhs) | CmpIndices::new(rhs)).as_ref()
            );
        }

        #[test]
        fn container_unique() {
            let lhs = KeyIndices::new(5);

            let rhs = KeyIndices::new(5);
            assert_eq!([5], (CmpIndices::new(lhs) | CmpIndices::new(rhs)).as_ref());

            let mut lhs = KeyIndices::new(5);
            lhs.add(0);
            let rhs = KeyIndices::new(5);
            assert_eq!(
                [0, 5],
                (CmpIndices::new(lhs) | CmpIndices::new(rhs)).as_ref()
            );
        }

        #[test]
        fn remove() {
            let mut pos = KeyIndices::new(5);
            assert_eq!([5], pos.as_ref());

            assert!(pos.remove(5).is_empty());
            // double remove
            assert!(pos.remove(5).is_empty());

            let mut pos = KeyIndices::new(5);
            pos.add(2);
            assert_eq!([2], pos.remove(5).as_ref());

            let mut pos = KeyIndices::new(5);
            pos.add(2);
            assert_eq!([5], pos.remove(2).as_ref());
        }
    }

    mod indices_or {
        use super::*;

        // Indices - ORs:
        // left | right
        // expected
        #[rstest]
        #[case::empty(Indices::empty(), Indices::empty(), Indices::empty())]
        #[case::only_left(
            Indices::borrowed(&[1, 2]), Indices::empty(),
            Indices::borrowed(&[1, 2]),
        )]
        #[case::only_right(
            Indices::empty(), Indices::borrowed(&[1, 2]),
            Indices::borrowed(&[1, 2]),
        )]
        #[case::diff_len1(
            Indices::borrowed(&[1]), Indices::borrowed(&[2, 3]),
            Indices::borrowed(&[1, 2, 3]),
        )]
        #[case::diff_len2(
            Indices::borrowed(&[2, 3]), Indices::borrowed(&[1]),
            Indices::borrowed(&[1, 2, 3]),
        )]
        #[case::overlapping_simple1(
            Indices::borrowed(&[1, 2]), Indices::borrowed(&[2, 3]),
            Indices::borrowed(&[1, 2, 3]),
        )]
        #[case::overlapping_simple2(
            Indices::borrowed(&[2, 3]), Indices::borrowed(&[1, 2]),
            Indices::borrowed(&[1, 2, 3]),
        )]
        #[case::overlapping_diff_len1(
            // 1, 2, 8, 9, 12
            // 2, 5, 6, 10
            Indices::borrowed(&[1, 2, 8, 9, 12]), Indices::borrowed(&[2, 5, 6, 10]),
            Indices::borrowed(&[1, 2, 5, 6, 8, 9, 10, 12]),
        )]
        #[case::overlapping_diff_len1(
            // 2, 5, 6, 10
            // 1, 2, 8, 9, 12
            Indices::borrowed(&[2, 5, 6, 10]), Indices::borrowed(&[1, 2, 8, 9, 12]),
            Indices::borrowed(&[1, 2, 5, 6, 8, 9, 10, 12]),
        )]
        fn ors(#[case] left: Indices, #[case] right: Indices, #[case] expected: Indices) {
            let left: CmpIndices = left.into();
            let right: CmpIndices = right.into();
            let expected: CmpIndices = expected.into();

            assert_eq!(expected.as_ref(), (left | right).as_ref());
        }
    }

    mod indices_and {
        use super::*;

        // Indices - ANDs:
        // left | right
        // expected
        #[rstest]
        #[case::empty(Indices::empty(), Indices::empty(), Indices::empty())]
        #[case::only_left(Indices::borrowed(&[1, 2]), Indices::empty(), Indices::empty())]
        #[case::only_right(Indices::empty(), Indices::borrowed(&[1, 2]), Indices::empty())]
        #[case::overlapping(Indices::borrowed(&[2, 3]), Indices::borrowed(&[1, 2]), Indices::borrowed(&[2]))]
        fn ands(#[case] left: Indices, #[case] right: Indices, #[case] expected: Indices) {
            let left: CmpIndices = left.into();
            let right: CmpIndices = right.into();
            let expected: CmpIndices = expected.into();

            assert_eq!(expected.as_ref(), (left & right).as_ref());
        }

        #[test]
        fn diff_len() {
            assert_eq!(
                CmpIndices::borrowed(&[]),
                CmpIndices::borrowed(&[1]) & CmpIndices::borrowed(&[2, 3])
            );
            assert_eq!(
                CmpIndices::borrowed(&[]),
                CmpIndices::borrowed(&[2, 3]) & CmpIndices::borrowed(&[1])
            );

            assert_eq!(
                CmpIndices::borrowed(&[2]),
                CmpIndices::borrowed(&[2]) & CmpIndices::borrowed(&[2, 5])
            );
            assert_eq!(
                CmpIndices::borrowed(&[2]),
                CmpIndices::borrowed(&[2]) & CmpIndices::borrowed(&[1, 2, 3])
            );
            assert_eq!(
                CmpIndices::borrowed(&[2]),
                CmpIndices::borrowed(&[2]) & CmpIndices::borrowed(&[0, 1, 2])
            );

            assert_eq!(
                CmpIndices::borrowed(&[2]),
                CmpIndices::borrowed(&[2, 5]) & CmpIndices::borrowed(&[2])
            );
            assert_eq!(
                CmpIndices::borrowed(&[2]),
                CmpIndices::borrowed(&[1, 2, 3]) & CmpIndices::borrowed(&[2])
            );
            assert_eq!(
                CmpIndices::borrowed(&[2]),
                CmpIndices::borrowed(&[0, 1, 2]) & CmpIndices::borrowed(&[2])
            );
        }

        #[test]
        fn overlapping_simple() {
            assert_eq!(
                CmpIndices::borrowed(&[2]),
                CmpIndices::borrowed(&[1, 2]) & CmpIndices::borrowed(&[2, 3]),
            );
            assert_eq!(
                CmpIndices::borrowed(&[2]),
                CmpIndices::borrowed(&[2, 3]) & CmpIndices::borrowed(&[1, 2]),
            );

            assert_eq!(
                CmpIndices::borrowed(&[1]),
                CmpIndices::borrowed(&[1, 2]) & CmpIndices::borrowed(&[1, 3]),
            );
            assert_eq!(
                CmpIndices::borrowed(&[1]),
                CmpIndices::borrowed(&[1, 3]) & CmpIndices::borrowed(&[1, 2]),
            );
        }

        #[test]
        fn overlapping_diff_len() {
            // 1, 2, 8, 9, 12
            // 2, 5, 6, 10
            assert_eq!(
                CmpIndices::borrowed(&[2, 12]),
                CmpIndices::borrowed(&[1, 2, 8, 9, 12])
                    & CmpIndices::borrowed(&[2, 5, 6, 10, 12, 13, 15])
            );

            // 2, 5, 6, 10
            // 1, 2, 8, 9, 12
            assert_eq!(
                CmpIndices::borrowed(&[2, 12]),
                CmpIndices::borrowed(&[2, 5, 6, 10, 12, 13, 15])
                    & CmpIndices::borrowed(&[1, 2, 8, 9, 12])
            );
        }
    }

    mod indices_query {
        use super::*;

        struct List(Vec<usize>);

        impl List {
            fn eq(&self, i: usize) -> CmpIndices<'_> {
                match self.0.binary_search(&i) {
                    Ok(pos) => CmpIndices::owned(vec![pos]),
                    Err(_) => Indices::empty().into(),
                }
            }
        }

        fn values() -> List {
            List(vec![0, 1, 2, 3])
        }

        #[test]
        fn filter() {
            let l = values();
            assert_eq!(1, l.eq(1)[0]);
            assert_eq!(Indices::empty().as_ref(), values().eq(99).as_ref());
        }

        #[test]
        fn and() {
            let l = values();
            assert_eq!(1, (l.eq(1) & l.eq(1))[0]);
            assert_eq!(Indices::empty().as_ref(), (l.eq(1) & l.eq(2)).as_ref());
        }

        #[test]
        fn or() {
            let l = values();
            assert_eq!([1, 2], (l.eq(1) | l.eq(2)).as_ref());
            assert_eq!([1], (l.eq(1) | l.eq(99)).as_ref());
            assert_eq!([1], (l.eq(99) | l.eq(1)).as_ref());
        }

        #[test]
        fn and_or() {
            let l = values();
            // (1 and 1) or 2 => [1, 2]
            assert_eq!([1, 2], (l.eq(1) & l.eq(1) | l.eq(2)).as_ref());
            // (1 and 2) or 3 => [3]
            assert_eq!([3], (l.eq(1) & l.eq(2) | l.eq(3)).as_ref());
        }

        #[test]
        fn or_and_12() {
            let l = values();
            // 1 or (2 and 2) => [1, 2]
            assert_eq!([1, 2], (l.eq(1) | l.eq(2) & l.eq(2)).as_ref());
            // 1 or (3 and 2) => [1]
            assert_eq!([1], (l.eq(1) | l.eq(3) & l.eq(2)).as_ref());
        }

        #[test]
        fn or_and_3() {
            let l = values();
            // 3 or (2 and 1) => [3]
            assert_eq!([3], (l.eq(3) | l.eq(2) & l.eq(1)).as_ref());
        }

        #[test]
        fn and_or_and_2() {
            let l = values();
            // (2 and 2) or (2 and 1) => [2]
            assert_eq!([2], (l.eq(2) & l.eq(2) | l.eq(2) & l.eq(1)).as_ref());
        }

        #[test]
        fn and_or_and_03() {
            let l = values();
            // 0 or (1 and 2) or 3) => [0, 3]
            assert_eq!([0, 3], (l.eq(0) | l.eq(1) & l.eq(2) | l.eq(3)).as_ref());
        }

        #[test]
        fn iter() {
            let idxs = CmpIndices::owned(vec![1, 3, 2]);
            let mut it = idxs.iter();
            assert_eq!(Some(&1), it.next());
            assert_eq!(Some(&3), it.next());
            assert_eq!(Some(&2), it.next());
        }
    }

    #[test]
    fn filter_get() {
        let list = StrIndex::new();
        assert!(list.contains(&"a"));
        assert_eq!([1], *list.get(&"b"));
        assert_eq!([2], *list.get(&"c"));
        assert_eq!(Indices::empty(), list.get(&"zz"));
    }

    #[test]
    fn filter_eq() {
        let list = StrIndex::new();
        let f = Filter(&list);

        assert_eq!([1, 2], *(f.eq(&"c") | f.eq(&"b")));
        assert_eq!([0, 1, 3], *(f.eq(&"a") | f.eq(&"b")));
        assert_eq!([0, 3], *(f.eq(&"a") | f.eq(&"a")));
        assert_eq!([0, 1, 3], *(f.eq(&"a") | f.eq(&"b") | f.eq(&"z")));

        assert_eq!(Vec::<usize>::new(), *(f.eq(&"c") & f.eq(&"b")));
    }

    #[test]
    fn filter_get_many() {
        let list = StrIndex::new();
        assert_eq!(
            [&"a", &"a", &"b"],
            *list.get_many(["a", "b"]).items_vec(&values)
        );
        assert_eq!([&"b"], *list.get_many(["z", "b", "y"]).items_vec(&values));
        assert_eq!(
            Vec::<&&str>::new(),
            list.get_many(["z", "y"]).items_vec(&values)
        );
    }

    #[test]
    fn iter_many() {
        let list = StrIndex::new();
        let mut many = Many::new(&list, [""].into_iter());
        assert_eq!(None, many.next());

        let many = Many::new(&list, [""].into_iter());
        assert_eq!(None, many.items(&values).next());

        let mut many = Many::new(&list, ["c"].into_iter());
        assert_eq!(Some(&2), many.next());

        let many = Many::new(&list, ["c"].into_iter());
        assert_eq!(Some(&"c"), many.items(&values).next());
    }

    #[test]
    fn extend_filter() {
        let list = StrIndex::new();
        let f = Filter(&list);

        assert_eq!([0, 2, 3], *f.or(&"c", &"a"));
        assert_eq!([0, 3], *f.or(&"zz", &"a"));

        assert_eq!([2], *extended_filter(&f, &"c"));
    }

    #[test]
    fn cmp_indices() {
        let idx: CmpIndices = vec![3, 2, 4, 1].into();
        assert_eq!([1, 2, 3, 4], idx.as_ref());
    }
}
