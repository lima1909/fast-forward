//! The purpose of an Index is to find faster a specific item in a list (Slice, Vec, ...).
//! This means, it does not have to touch and compare every item in the list.
//!
//! An Index has two parts, a `Key` (item to search for) and a position (the index in the list) `Index`.
//!
//! There are two types of Index:
//! - `Unique Index`: for a given `Key` exist exactly one Index.
//! - `Multi Index` : for a given `Key` exists many Indices.
//!
//! # Example for an Vec-Multi-Index:
//!
//! Map-Index:
//!
//! - `Key`   = name (String)
//! - `Index` = index is the position in a List (Vec)
//!
//! ```text
//! let _names = vec!["Paul", "Jasmin", "Inge", "Paul", ...];
//!
//!  Key       | Index
//! -------------------
//!  "Jasmin"  | 1
//!  "Paul"    | 0, 3
//!  "Inge"    | 2
//!   ...      | ...
//! ```
pub mod map;
pub mod store;
pub mod uint;

pub use store::{EqFilter, ItemRetriever, NoMeta, Retriever, Store};

use std::{
    borrow::Cow,
    cmp::{min, Ordering::*},
    ops::{BitAnd, BitOr, Index},
    slice,
};

#[derive(Debug, Default, Clone)]
#[repr(transparent)]
pub struct SelectedIndices<'i>(Cow<'i, [usize]>);

/// `SelIdx` (Selected Indices) is the result from quering (filter) a list.
impl<'i> SelectedIndices<'i> {
    #[inline]
    pub fn new(i: usize) -> Self {
        Self(Cow::Owned(vec![i]))
    }

    pub const fn empty() -> Self {
        Self(Cow::Owned(Vec::new()))
    }

    pub const fn borrowed(s: &'i [usize]) -> Self {
        Self(Cow::Borrowed(s))
    }

    pub const fn owned(v: Vec<usize>) -> Self {
        Self(Cow::Owned(v))
    }

    #[inline]
    pub fn iter(&self) -> slice::Iter<'_, usize> {
        self.0.iter()
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

impl<'i> Index<usize> for SelectedIndices<'i> {
    type Output = usize;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<'i, const N: usize> PartialEq<[usize; N]> for SelectedIndices<'i> {
    fn eq(&self, other: &[usize; N]) -> bool {
        (*self.0).eq(other)
    }
}

impl<'i, const N: usize> PartialEq<SelectedIndices<'i>> for [usize; N] {
    fn eq(&self, other: &SelectedIndices) -> bool {
        (self).eq(&*other.0)
    }
}

impl<'i> PartialEq for SelectedIndices<'i> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<'i> BitOr for SelectedIndices<'i> {
    type Output = Self;

    fn bitor(self, other: Self) -> Self::Output {
        let lhs = &self.0;
        let rhs = &other.0;

        match (lhs.is_empty(), rhs.is_empty()) {
            (false, false) => {
                let (ll, lr) = (lhs.len(), rhs.len());
                let mut v = Vec::with_capacity(ll + lr);

                let (mut li, mut ri) = (0, 0);

                loop {
                    let (l, r) = (lhs[li], rhs[ri]);

                    match l.cmp(&r) {
                        Equal => {
                            v.push(l);
                            li += 1;
                            ri += 1;
                        }
                        Less => {
                            v.push(l);
                            li += 1;
                        }
                        Greater => {
                            v.push(r);
                            ri += 1;
                        }
                    }

                    if ll == li {
                        v.extend(rhs[ri..].iter());
                        return SelectedIndices::owned(v);
                    } else if lr == ri {
                        v.extend(lhs[li..].iter());
                        return SelectedIndices::owned(v);
                    }
                }
            }
            (true, false) => other,
            (false, true) => self,
            (true, true) => SelectedIndices::empty(),
        }
    }
}

impl<'i> BitAnd for SelectedIndices<'i> {
    type Output = Self;

    fn bitand(self, other: Self) -> Self::Output {
        let lhs = &self.0;
        let rhs = &other.0;

        if lhs.is_empty() || rhs.is_empty() {
            return SelectedIndices::empty();
        }

        let (ll, lr) = (lhs.len(), rhs.len());
        let mut v = Vec::with_capacity(min(ll, lr));

        let (mut li, mut ri) = (0, 0);

        loop {
            let l = lhs[li];

            match l.cmp(&rhs[ri]) {
                Equal => {
                    v.push(l);
                    li += 1;
                    ri += 1;
                }
                Less => li += 1,
                Greater => ri += 1,
            }

            if li == ll || ri == lr {
                return SelectedIndices::owned(v);
            }
        }
    }
}

/// `Indices` is a wrapper for saving all indices for a given `Key` in the `Store`.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Indices(Vec<usize>);

impl Indices {
    /// Create a new Indices collection with the initial Index.
    #[inline]
    pub fn new(idx: usize) -> Self {
        Self(vec![idx])
    }

    /// Add a new Index.
    #[inline]
    pub fn add(&mut self, idx: usize) {
        if let Err(pos) = self.0.binary_search(&idx) {
            self.0.insert(pos, idx);
        }
    }

    /// Return all saved Indices and return as `SelIdx` object.
    #[inline]
    pub fn get(&self) -> SelectedIndices<'_> {
        SelectedIndices::borrowed(&self.0)
    }

    /// Remove one Index and return left free Indices.
    #[inline]
    pub fn remove(&mut self, idx: usize) -> SelectedIndices<'_> {
        self.0.retain(|v| v != &idx);
        self.get()
    }
}

#[derive(Debug, Default)]
struct MinMax<K> {
    min: K,
    max: K,
}

impl<K: Default + Ord> MinMax<K> {
    fn new_min_value(&mut self, key: K) -> &K {
        if self.min == K::default() || self.min > key {
            self.min = key
        }
        &self.min
    }

    fn new_max_value(&mut self, key: K) -> &K {
        if self.max < key {
            self.max = key
        }
        &self.max
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod selected_indices {
        use super::*;

        impl<'i> SelectedIndices<'i> {
            fn from_slice(s: &'i [usize]) -> Self {
                Self(Cow::Borrowed(s))
            }
        }

        mod indices_or {
            use super::*;

            #[test]
            fn both_empty() {
                assert_eq!(
                    SelectedIndices::empty(),
                    SelectedIndices::empty() | SelectedIndices::empty()
                );
            }

            #[test]
            fn only_left() {
                assert_eq!(
                    SelectedIndices::from_slice(&[1, 2]),
                    SelectedIndices::from_slice(&[1, 2]) | SelectedIndices::empty()
                );
            }

            #[test]
            fn only_right() {
                assert_eq!(
                    SelectedIndices::from_slice(&[1, 2]),
                    SelectedIndices::empty() | SelectedIndices::from_slice(&[1, 2])
                );
            }

            #[test]
            fn diff_len() {
                assert_eq!(
                    SelectedIndices::from_slice(&[1, 2, 3]),
                    SelectedIndices::new(1) | SelectedIndices::from_slice(&[2, 3]),
                );
                assert_eq!(
                    SelectedIndices::from_slice(&[1, 2, 3]),
                    SelectedIndices::from_slice(&[2, 3]) | SelectedIndices::new(1)
                );
            }

            #[test]
            fn overlapping_simple() {
                assert_eq!(
                    SelectedIndices::from_slice(&[1, 2, 3]),
                    SelectedIndices::from_slice(&[1, 2]) | SelectedIndices::from_slice(&[2, 3])
                );
                assert_eq!(
                    SelectedIndices::from_slice(&[1, 2, 3]),
                    SelectedIndices::from_slice(&[2, 3]) | SelectedIndices::from_slice(&[1, 2])
                );
            }

            #[test]
            fn overlapping_diff_len() {
                // 1, 2, 8, 9, 12
                // 2, 5, 6, 10
                assert_eq!(
                    SelectedIndices::from_slice(&[1, 2, 5, 6, 8, 9, 10, 12]),
                    SelectedIndices::from_slice(&[1, 2, 8, 9, 12])
                        | SelectedIndices::from_slice(&[2, 5, 6, 10])
                );

                // 2, 5, 6, 10
                // 1, 2, 8, 9, 12
                assert_eq!(
                    SelectedIndices::from_slice(&[1, 2, 5, 6, 8, 9, 10, 12]),
                    SelectedIndices::from_slice(&[2, 5, 6, 10])
                        | SelectedIndices::from_slice(&[1, 2, 8, 9, 12])
                );
            }
        }

        mod indices_and {
            use super::*;

            #[test]
            fn both_empty() {
                assert_eq!(
                    SelectedIndices::empty(),
                    SelectedIndices::empty() & SelectedIndices::empty()
                );
            }

            #[test]
            fn only_left() {
                assert_eq!(
                    SelectedIndices::empty(),
                    SelectedIndices::from_slice(&[1, 2]) & SelectedIndices::empty()
                );
            }

            #[test]
            fn only_right() {
                assert_eq!(
                    SelectedIndices::empty(),
                    SelectedIndices::empty() & SelectedIndices::from_slice(&[1, 2])
                );
            }

            #[test]
            fn diff_len() {
                assert_eq!(
                    SelectedIndices::empty(),
                    SelectedIndices::from_slice(&[1]) & SelectedIndices::from_slice(&[2, 3])
                );
                assert_eq!(
                    SelectedIndices::empty(),
                    SelectedIndices::from_slice(&[2, 3]) & SelectedIndices::from_slice(&[1])
                );

                assert_eq!(
                    [2],
                    SelectedIndices::from_slice(&[2]) & SelectedIndices::from_slice(&[2, 5])
                );
                assert_eq!(
                    [2],
                    SelectedIndices::from_slice(&[2]) & SelectedIndices::from_slice(&[1, 2, 3])
                );
                assert_eq!(
                    [2],
                    SelectedIndices::from_slice(&[2]) & SelectedIndices::from_slice(&[0, 1, 2])
                );

                assert_eq!(
                    [2],
                    SelectedIndices::from_slice(&[2, 5]) & SelectedIndices::from_slice(&[2])
                );
                assert_eq!(
                    [2],
                    SelectedIndices::from_slice(&[1, 2, 3]) & SelectedIndices::from_slice(&[2])
                );
                assert_eq!(
                    [2],
                    SelectedIndices::from_slice(&[0, 1, 2]) & SelectedIndices::from_slice(&[2])
                );
            }

            #[test]
            fn overlapping_simple() {
                assert_eq!(
                    [2],
                    SelectedIndices::from_slice(&[1, 2]) & SelectedIndices::from_slice(&[2, 3]),
                );
                assert_eq!(
                    [2],
                    SelectedIndices::from_slice(&[2, 3]) & SelectedIndices::from_slice(&[1, 2]),
                );

                assert_eq!(
                    [1],
                    SelectedIndices::from_slice(&[1, 2]) & SelectedIndices::from_slice(&[1, 3]),
                );
                assert_eq!(
                    [1],
                    SelectedIndices::from_slice(&[1, 3]) & SelectedIndices::from_slice(&[1, 2]),
                );
            }

            #[test]
            fn overlapping_diff_len() {
                // 1, 2, 8, 9, 12
                // 2, 5, 6, 10
                assert_eq!(
                    [2, 12],
                    SelectedIndices::from_slice(&[1, 2, 8, 9, 12])
                        & SelectedIndices::from_slice(&[2, 5, 6, 10, 12, 13, 15])
                );

                // 2, 5, 6, 10
                // 1, 2, 8, 9, 12
                assert_eq!(
                    [2, 12],
                    SelectedIndices::from_slice(&[2, 5, 6, 10, 12, 13, 15])
                        & SelectedIndices::from_slice(&[1, 2, 8, 9, 12])
                );
            }
        }

        mod query {
            use super::*;

            struct List(Vec<usize>);

            impl List {
                fn eq(&self, i: usize) -> SelectedIndices<'_> {
                    match self.0.binary_search(&i) {
                        Ok(pos) => SelectedIndices::owned(vec![pos]),
                        Err(_) => SelectedIndices::empty(),
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
                assert_eq!(SelectedIndices::empty(), values().eq(99));
            }

            #[test]
            fn and() {
                let l = values();
                assert_eq!(1, (l.eq(1) & l.eq(1))[0]);
                assert_eq!(SelectedIndices::empty(), (l.eq(1) & l.eq(2)));
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
        }
    }

    mod indices {
        use super::*;

        #[test]
        fn unique() {
            let u = Indices::new(0);
            assert_eq!([0], u.get());
        }

        #[test]
        fn multi() {
            let mut m = Indices::new(2);
            assert_eq!([2], m.get());

            m.add(1);
            assert_eq!([1, 2], m.get());
        }

        #[test]
        fn multi_duplicate() {
            let mut m = Indices::new(1);
            assert_eq!([1], m.get());

            // ignore add: 1, 1 exists already
            m.add(1);
            assert_eq!([1], m.get());
        }

        #[test]
        fn multi_ordered() {
            let mut m = Indices::new(5);
            assert_eq!([5], m.get());

            m.add(3);
            m.add(1);
            m.add(4);

            assert_eq!([1, 3, 4, 5], m.get());
        }

        #[test]
        fn container_multi() {
            let mut lhs = Indices::new(5);
            lhs.add(3);
            lhs.add(2);
            lhs.add(4);

            let mut rhs = Indices::new(5);
            rhs.add(2);
            rhs.add(9);

            assert_eq!([2, 3, 4, 5, 9], lhs.get() | rhs.get());
        }

        #[test]
        fn container_unique() {
            let mut lhs = Indices::new(5);

            let rhs = Indices::new(5);
            assert_eq!([5], lhs.get() | rhs.get());

            lhs.add(0);
            assert_eq!([0, 5], lhs.get() | rhs.get());
        }

        #[test]
        fn remove() {
            let mut pos = Indices::new(5);
            assert_eq!([5], pos.get());

            assert!(pos.remove(5).is_empty());
            // double remove
            assert!(pos.remove(5).is_empty());

            let mut pos = Indices::new(5);
            pos.add(2);
            assert_eq!([2], pos.remove(5));

            let mut pos = Indices::new(5);
            pos.add(2);
            assert_eq!([5], pos.remove(2));
        }
    }

    mod min_max {
        use super::*;

        #[test]
        fn min() {
            assert_eq!(0, MinMax::default().min);
            assert_eq!(&0, MinMax::default().new_min_value(0));
            assert_eq!(&1, MinMax::default().new_min_value(1));

            let mut min = MinMax::default();
            min.new_min_value(1);
            min.new_min_value(0);
            assert_eq!(0, min.min);

            let mut min = MinMax::default();
            min.new_min_value(1);
            min.new_min_value(2);
            assert_eq!(1, min.min);

            let mut min = MinMax::default();
            min.new_min_value(2);
            min.new_min_value(1);
            assert_eq!(1, min.min);
        }

        #[test]
        fn max() {
            assert_eq!(0, MinMax::default().max);
            assert_eq!(&0, MinMax::default().new_max_value(0));
            assert_eq!(&1, MinMax::default().new_max_value(1));

            let mut max = MinMax::default();
            max.new_max_value(1);
            max.new_max_value(0);
            assert_eq!(1, max.max);

            let mut max = MinMax::default();
            max.new_max_value(1);
            max.new_max_value(2);
            assert_eq!(2, max.max);
        }
    }
}
