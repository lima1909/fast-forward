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

use crate::SelectedIndices;

pub use store::{EqFilter, ItemRetriever, NoMeta, Retriever, Store};

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
        (&self.0).into()
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

    #[test]
    fn index_unique() {
        let u = Indices::new(0);
        assert_eq!([0], u.get());
    }

    #[test]
    fn index_multi() {
        let mut m = Indices::new(2);
        assert_eq!([2], m.get());

        m.add(1);
        assert_eq!([1, 2], m.get());
    }

    #[test]
    fn index_multi_duplicate() {
        let mut m = Indices::new(1);
        assert_eq!([1], m.get());

        // ignore add: 1, 1 exists already
        m.add(1);
        assert_eq!([1], m.get());
    }

    #[test]
    fn index_multi_ordered() {
        let mut m = Indices::new(5);
        assert_eq!([5], m.get());

        m.add(3);
        m.add(1);
        m.add(4);

        assert_eq!([1, 3, 4, 5], m.get());
    }

    #[test]
    fn index_container_multi() {
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
    fn index_container_unique() {
        let mut lhs = Indices::new(5);

        let rhs = Indices::new(5);
        assert_eq!([5], lhs.get() | rhs.get());

        lhs.add(0);
        assert_eq!([0, 5], lhs.get() | rhs.get());
    }

    #[test]
    fn index_remove() {
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
