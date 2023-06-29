use std::{
    borrow::Cow,
    ops::{BitAnd, BitOr, Deref, Index},
};

pub trait Filterable {
    type Key;

    /// Get all indices for a given `Key`.
    /// If the `Key` not exist, than this method returns [`Indices::empty()`]
    fn get<'a>(&'a self, key: &Self::Key) -> Indices<'a>;

    /// ???
    fn get_cmp<'a>(&'a self, key: &Self::Key) -> CmpIndices<'a> {
        CmpIndices::indices(self.get(key))
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

    /// Checks whether the `Key` exists.
    #[inline]
    fn contains(&self, key: &Self::Key) -> bool {
        !self.get(key).is_empty()
    }
}

/// Is using from the [`crate::index::Store`] to save the `Indices` for a given `Key`.
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

/// Is a wrapper for selected `Indices`, e.g. by using the [`Filterable`] trait, the `get` method.
/// You can create an instance with: [`Indices::empty()`] or `KeyIndices::into()` (ordered list of indices).
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

#[repr(transparent)]
pub struct CmpIndices<'i>(Cow<'i, [usize]>);

impl<'i> CmpIndices<'i> {
    pub fn indices(Indices(idx): Indices<'i>) -> Self {
        Self(Cow::Borrowed(idx))
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

#[cfg(test)]
mod tests {
    use super::*;

    use std::{collections::HashMap, ops::Deref};

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

    struct Filter<'f, F: Filterable>(&'f F);

    impl<'f, F: Filterable> Deref for Filter<'f, F> {
        type Target = F;

        fn deref(&self) -> &Self::Target {
            self.0
        }
    }

    trait Or<'f> {
        type Key;

        fn or(&'f self, key1: &Self::Key, key2: &Self::Key) -> CmpIndices<'f>;
    }

    impl<'f, F: Filterable> Or<'f> for Filter<'f, F> {
        type Key = F::Key;

        fn or(&'f self, key1: &Self::Key, key2: &Self::Key) -> CmpIndices<'f> {
            self.get_cmp(key1) | self.get_cmp(key2)
        }
    }

    fn extended_filter<'i>(f: &'i Filter<'i, StrIndex>, key: &'static &str) -> Indices<'i> {
        f.0.get(key)
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
    fn filter_get_cmp() {
        let list = StrIndex::new();

        assert_eq!([1, 2], *(list.get_cmp(&"c") | list.get_cmp(&"b")));
        assert_eq!([0, 1, 3], *(list.get_cmp(&"a") | list.get_cmp(&"b")));
        assert_eq!([0, 3], *(list.get_cmp(&"a") | list.get_cmp(&"a")));
        assert_eq!(
            [0, 1, 3],
            *(list.get_cmp(&"a") | list.get_cmp(&"b") | list.get_cmp(&"z"))
        );
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

    // #[rstest]
    // #[case::empty(vec![], vec![])]
    // #[case::one_found(vec!["c"], vec![&"c"])]
    // #[case::one_not_found(vec!["-"], vec![])]
    // #[case::m_z_a(vec!["m", "z", "a"], vec![&"z", &"a"])]
    // #[case::a_m_z(vec![ "a","m", "z"], vec![&"a", &"z"])]
    // #[case::z_m_a(vec![ "z","m", "a"], vec![&"z", &"a"])]
    // #[case::m_z_a_m(vec!["m", "z", "a", "m"], vec![&"z", &"a"])]
    // #[case::m_z_a_m_m(vec!["m", "z", "a", "m", "m"], vec![&"z", &"a"])]
    // #[case::double_x(vec!["x"], vec![&"x", &"x"])]
    // #[case::a_double_x(vec!["a", "x"], vec![&"a", &"x", &"x"])]
    // fn view_str(#[case] keys: Vec<&str>, #[case] expected: Vec<&&str>) {
    //     let items = vec!["x", "a", "b", "c", "x", "y", "z"];
    //     let map = MapIndex::from_iter(items.clone().into_iter());
    //     assert_eq!(expected, map.get_many(keys).items_vec(&items));
    // }
}
