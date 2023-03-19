use std::{fmt::Debug, marker::PhantomData};

use crate::{error::Error, Idx, Result};

/// This trait descripe the interface for an possible Index.
pub trait Index: Debug {
    /// Create a new `Index`.
    fn new(i: Idx) -> Self;
    /// Add an `Index`.
    fn add(&mut self, i: Idx) -> Result;
    /// Get all `Indecies`. **Importand:** The items must be sorted in ascending order!
    fn get(&self) -> &[Idx];
}

/// A unique `Index`. This means, it contains only one `Index` for a given `Key`.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Unique([Idx; 1]);

impl Index for Unique {
    #[inline]
    fn new(i: Idx) -> Self {
        Unique([i])
    }

    #[inline]
    fn add(&mut self, _i: Idx) -> Result {
        Err(Error::NotUniqueIndexKey)
    }

    #[inline]
    fn get(&self) -> &[Idx] {
        &self.0
    }
}

/// A multi `Index`. This means, it contains many `Indices` for a given `Key`.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Multi(Vec<Idx>);

impl Index for Multi {
    #[inline]
    fn new(i: Idx) -> Self {
        Multi(vec![i])
    }

    #[inline]
    fn add(&mut self, i: Idx) -> Result {
        if let Err(pos) = self.0.binary_search(&i) {
            self.0.insert(pos, i);
        }
        Ok(())
    }

    #[inline]
    fn get(&self) -> &[Idx] {
        &self.0
    }
}

/// Positions is an container for gathering [`Index`] values (&[Idx]).
/// It is usefull for operations like greater then ([`crate::Op::GT`]),
/// where the result consists one or many [`Index`]s.
pub struct Positions<I>(Vec<Idx>, PhantomData<I>);

impl<I: Index> Positions<I> {
    #[inline]
    pub fn new(i: I) -> Self {
        Positions(Vec::from_iter(i.get().iter().copied()), PhantomData)
    }

    #[inline]
    pub fn add(&mut self, i: I) {
        self.0.extend(i.get());
    }

    #[inline]
    pub fn get(&self) -> &[Idx] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique() {
        let mut u = Unique::new(0);
        assert_eq!(&[0], u.get());

        let err = u.add(1);
        assert!(err.is_err());

        assert_eq!(1, u.get().len());
    }

    #[test]
    fn multi() {
        let mut m = Multi::new(0);
        assert_eq!(&[0], m.get());

        m.add(1).unwrap();
        assert_eq!(2, m.get().len());
        assert_eq!(&[0, 1], m.get());
    }

    #[test]
    fn multi_duplicate() {
        let mut m = Multi::new(1);
        assert_eq!(&[1], m.get());

        // ignore add: 1, 1 exists already
        m.add(1).unwrap();
        assert_eq!(1, m.get().len());
        assert_eq!(&[1], m.get());
    }

    #[test]
    fn multi_ordered() {
        let mut m = Multi::new(5);
        assert_eq!(&[5], m.get());

        m.add(3).unwrap();
        m.add(1).unwrap();
        m.add(4).unwrap();

        assert_eq!(&[1, 3, 4, 5], m.get());
    }
}
