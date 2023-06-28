//! There are two kind of `Indices`
//! - KeyIndices: is a collection of all `Indices`for a given `Key`
//! - Indices: is a collection (read only) of selected `Indices`
use std::{
    borrow::Cow,
    ops::{BitAnd, BitOr, Index},
    slice,
};

/// `KeyIndices` contains all indices for a given `Key`.
/// Important: the collection must be sorted!
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct KeyIndices(Vec<usize>);

impl KeyIndices {
    /// Create a new Indices collection with the initial Index.
    #[inline]
    pub fn new(idx: usize) -> Self {
        Self(vec![idx])
    }

    /// Add new Index to a sorted collection.
    #[inline]
    pub fn add(&mut self, idx: usize) {
        if let Err(pos) = self.0.binary_search(&idx) {
            self.0.insert(pos, idx);
        }
    }

    /// Remove one Index and return left free Indices.
    #[inline]
    pub fn remove(&mut self, idx: usize) -> Indices<'_> {
        self.0.retain(|v| v != &idx);
        self.indices()
    }

    // ???
    #[inline]
    pub fn iter(&self) -> slice::Iter<'_, usize> {
        self.0.iter()
    }

    /// Return all collected Indices.
    #[inline]
    pub fn indices(&self) -> Indices<'_> {
        Indices(Cow::Borrowed(&self.0))
    }
}

impl<const N: usize> PartialEq<[usize; N]> for KeyIndices {
    fn eq(&self, other: &[usize; N]) -> bool {
        (*self.0).eq(other)
    }
}

impl<const N: usize> PartialEq<KeyIndices> for [usize; N] {
    fn eq(&self, other: &KeyIndices) -> bool {
        (self).eq(&*other.0)
    }
}

impl PartialEq for KeyIndices {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

/// `Indices` is a read only collection of selected Indices.
/// The `Indices` can be created as result from quering (filtering) a list.
#[derive(Debug)]
#[repr(transparent)]
pub struct Indices<'i>(Cow<'i, [usize]>);

impl<'i> Indices<'i> {
    #[inline]
    pub const fn empty() -> Self {
        Self(Cow::Borrowed(&[]))
    }

    // ???
    #[inline]
    pub fn iter(&self) -> slice::Iter<'_, usize> {
        self.0.iter()
    }

    // ???
    #[inline]
    pub fn items<I>(self, list: &'i I) -> Iter<'i, I>
    where
        I: Index<usize>,
    {
        Iter {
            pos: 0,
            list,
            indices: self,
        }
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&usize> {
        self.0.get(index)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

// ----------------------- ???
pub struct Iter<'i, I> {
    pos: usize,
    list: &'i I,
    indices: Indices<'i>,
}

impl<'i, I> Iterator for Iter<'i, I>
where
    I: Index<usize>,
{
    type Item = &'i I::Output;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.indices.get(self.pos)?;
        self.pos += 1;
        Some(&self.list[*idx])
    }
}

// ???
impl<'i, I> ExactSizeIterator for Iter<'i, I>
where
    I: Index<usize>,
{
    fn len(&self) -> usize {
        self.indices.len()
    }
}

impl Index<usize> for Indices<'_> {
    type Output = usize;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl From<usize> for Indices<'_> {
    fn from(u: usize) -> Self {
        Self(Cow::Owned(vec![u]))
    }
}

impl<const N: usize> From<[usize; N]> for Indices<'_> {
    fn from(mut s: [usize; N]) -> Self {
        s.sort();
        Self(Cow::Owned(Vec::from(s)))
    }
}

impl From<Vec<usize>> for Indices<'_> {
    fn from(mut v: Vec<usize>) -> Self {
        v.sort();
        Self(Cow::Owned(v))
    }
}

impl<const N: usize> PartialEq<[usize; N]> for Indices<'_> {
    fn eq(&self, other: &[usize; N]) -> bool {
        (*self.0).eq(other)
    }
}

impl PartialEq<Vec<usize>> for Indices<'_> {
    fn eq(&self, other: &Vec<usize>) -> bool {
        (*self.0).eq(other)
    }
}

impl PartialEq<Indices<'_>> for Vec<usize> {
    fn eq(&self, other: &Indices<'_>) -> bool {
        other.0.eq(self)
    }
}

impl<const N: usize> PartialEq<Indices<'_>> for [usize; N] {
    fn eq(&self, other: &Indices) -> bool {
        (self).eq(&*other.0)
    }
}

impl PartialEq for Indices<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl BitOr for Indices<'_> {
    type Output = Self;

    fn bitor(self, other: Self) -> Self::Output {
        Indices(super::union(self.0, other.0))
    }
}

impl BitAnd for Indices<'_> {
    type Output = Self;

    fn bitand(self, other: Self) -> Self::Output {
        Indices(super::intersection(self.0, other.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    impl<'i> Indices<'i> {
        const fn owned(v: Vec<usize>) -> Self {
            Self(Cow::Owned(v))
        }

        const fn borrowed(s: &'i [usize]) -> Self {
            Self(Cow::Borrowed(s))
        }
    }

    mod key_indices {
        use super::*;

        #[test]
        fn unique() {
            assert_eq!([0], KeyIndices::new(0));
        }

        #[test]
        fn multi() {
            let mut m = KeyIndices::new(2);
            assert_eq!([2], m);

            m.add(1);
            assert_eq!([1, 2], m);
        }

        #[test]
        fn multi_duplicate() {
            let mut m = KeyIndices::new(1);
            assert_eq!([1], m);

            // ignore add: 1, 1 exists already
            m.add(1);
            assert_eq!([1], m);
        }

        #[test]
        fn multi_ordered() {
            let mut m = KeyIndices::new(5);
            assert_eq!([5], m);

            m.add(3);
            m.add(1);
            m.add(4);

            assert_eq!([1, 3, 4, 5], m);
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

            assert_eq!([2, 3, 4, 5, 9], lhs.indices() | rhs.indices());
        }

        #[test]
        fn container_unique() {
            let mut lhs = KeyIndices::new(5);

            let rhs = KeyIndices::new(5);
            assert_eq!([5], lhs.indices() | rhs.indices());

            lhs.add(0);
            assert_eq!([0, 5], lhs.indices() | rhs.indices());
        }

        #[test]
        fn remove() {
            let mut pos = KeyIndices::new(5);
            assert_eq!([5], pos.indices());

            assert!(pos.remove(5).is_empty());
            // double remove
            assert!(pos.remove(5).is_empty());

            let mut pos = KeyIndices::new(5);
            pos.add(2);
            assert_eq!([2], pos.remove(5));

            let mut pos = KeyIndices::new(5);
            pos.add(2);
            assert_eq!([5], pos.remove(2));
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
            assert_eq!(expected, left | right);
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
            assert_eq!(expected, left & right);
        }

        #[test]
        fn diff_len() {
            assert_eq!([], Indices::borrowed(&[1]) & Indices::borrowed(&[2, 3]));
            assert_eq!([], Indices::borrowed(&[2, 3]) & Indices::borrowed(&[1]));

            assert_eq!([2], Indices::borrowed(&[2]) & Indices::borrowed(&[2, 5]));
            assert_eq!([2], Indices::borrowed(&[2]) & Indices::borrowed(&[1, 2, 3]));
            assert_eq!([2], Indices::borrowed(&[2]) & Indices::borrowed(&[0, 1, 2]));

            assert_eq!([2], Indices::borrowed(&[2, 5]) & Indices::borrowed(&[2]));
            assert_eq!([2], Indices::borrowed(&[1, 2, 3]) & Indices::borrowed(&[2]));
            assert_eq!([2], Indices::borrowed(&[0, 1, 2]) & Indices::borrowed(&[2]));
        }

        #[test]
        fn overlapping_simple() {
            assert_eq!([2], Indices::borrowed(&[1, 2]) & Indices::borrowed(&[2, 3]),);
            assert_eq!([2], Indices::borrowed(&[2, 3]) & Indices::borrowed(&[1, 2]),);

            assert_eq!([1], Indices::borrowed(&[1, 2]) & Indices::borrowed(&[1, 3]),);
            assert_eq!([1], Indices::borrowed(&[1, 3]) & Indices::borrowed(&[1, 2]),);
        }

        #[test]
        fn overlapping_diff_len() {
            // 1, 2, 8, 9, 12
            // 2, 5, 6, 10
            assert_eq!(
                [2, 12],
                Indices::borrowed(&[1, 2, 8, 9, 12])
                    & Indices::borrowed(&[2, 5, 6, 10, 12, 13, 15])
            );

            // 2, 5, 6, 10
            // 1, 2, 8, 9, 12
            assert_eq!(
                [2, 12],
                Indices::borrowed(&[2, 5, 6, 10, 12, 13, 15])
                    & Indices::borrowed(&[1, 2, 8, 9, 12])
            );
        }
    }

    mod indices_query {
        use super::*;

        struct List(Vec<usize>);

        impl List {
            fn eq(&self, i: usize) -> Indices<'_> {
                match self.0.binary_search(&i) {
                    Ok(pos) => Indices::owned(vec![pos]),
                    Err(_) => Indices::empty(),
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
            assert_eq!(Indices::empty(), values().eq(99));
        }

        #[test]
        fn and() {
            let l = values();
            assert_eq!(1, (l.eq(1) & l.eq(1))[0]);
            assert_eq!(Indices::empty(), (l.eq(1) & l.eq(2)));
        }

        #[test]
        fn or() {
            let l = values();
            assert_eq!([1, 2], l.eq(1) | l.eq(2));
            assert_eq!([1], l.eq(1) | l.eq(99));
            assert_eq!([1], l.eq(99) | l.eq(1));
        }

        #[test]
        fn and_or() {
            let l = values();
            // (1 and 1) or 2 => [1, 2]
            assert_eq!([1, 2], l.eq(1) & l.eq(1) | l.eq(2));
            // (1 and 2) or 3 => [3]
            assert_eq!([3], l.eq(1) & l.eq(2) | l.eq(3));
        }

        #[test]
        fn or_and_12() {
            let l = values();
            // 1 or (2 and 2) => [1, 2]
            assert_eq!([1, 2], l.eq(1) | l.eq(2) & l.eq(2));
            // 1 or (3 and 2) => [1]
            assert_eq!([1], l.eq(1) | l.eq(3) & l.eq(2));
        }

        #[test]
        fn or_and_3() {
            let l = values();
            // 3 or (2 and 1) => [3]
            assert_eq!([3], l.eq(3) | l.eq(2) & l.eq(1));
        }

        #[test]
        fn and_or_and_2() {
            let l = values();
            // (2 and 2) or (2 and 1) => [2]
            assert_eq!([2], l.eq(2) & l.eq(2) | l.eq(2) & l.eq(1));
        }

        #[test]
        fn and_or_and_03() {
            let l = values();
            // 0 or (1 and 2) or 3) => [0, 3]
            assert_eq!([0, 3], l.eq(0) | l.eq(1) & l.eq(2) | l.eq(3));
        }

        #[test]
        fn iter() {
            let idxs = Indices::owned(vec![1, 3, 2]);
            let mut it = idxs.iter();
            assert_eq!(Some(&1), it.next());
            assert_eq!(Some(&3), it.next());
            assert_eq!(Some(&2), it.next());
        }
    }
}
