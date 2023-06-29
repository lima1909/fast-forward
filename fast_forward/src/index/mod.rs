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
pub mod idx;
pub mod indices;
pub mod map;
pub mod store;
pub mod uint;

use std::{
    borrow::Cow,
    cmp::{min, Ordering::*},
};

pub use indices::{Indices, Iter, KeyIndices};
pub use store::{Filterable, MetaData, Store};

/// Union is using for OR
#[inline]
pub fn union<'c>(lhs: Cow<'c, [usize]>, rhs: Cow<'c, [usize]>) -> Cow<'c, [usize]> {
    if lhs.is_empty() {
        return rhs;
    }
    if rhs.is_empty() {
        return lhs;
    }

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
            v.extend(&rhs[ri..]);
            return Cow::Owned(v);
        } else if lr == ri {
            v.extend(&lhs[li..]);
            return Cow::Owned(v);
        }
    }
}

/// Intersection is using for AND
#[inline]
pub fn intersection<'c>(lhs: Cow<'c, [usize]>, rhs: Cow<'c, [usize]>) -> Cow<'c, [usize]> {
    if lhs.is_empty() {
        return lhs;
    }
    if rhs.is_empty() {
        return rhs;
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
            return Cow::Owned(v);
        }
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
