use index::{Idx, Indexer};

pub mod index;

/// Id for operations.
pub type Op = u8;

/// Filter is a given query K: Key (value) and operation: [`Op`]
pub struct Filter<K>(K, Op);

impl<K> Filter<K> {
    #[inline]
    pub fn key(&self) -> &K {
        &self.0
    }

    #[inline]
    pub fn op(&self) -> Op {
        self.1
    }
}

pub trait Query<K>: Indexer<K> + Sized {
    fn filter(&self, f: Filter<K>) -> &[Idx] {
        self.index(f)
    }

    fn or_rhs<'a, Rhs: Indexer<K>>(
        &'a self,
        l: Filter<K>,
        ridx: &'a Rhs,
        r: Filter<K>,
    ) -> Vec<&'a Idx> {
        ops::or(self, l, ridx, r)
    }

    fn or(&self, l: Filter<K>, r: Filter<K>) -> Vec<&Idx> {
        ops::or::<K, Self, Self>(self, l, self, r)
    }
}

impl<K, I: Indexer<K> + Sized> Query<K> for I {}

pub mod ops {
    use std::collections::HashSet;

    use crate::{
        index::{Idx, Indexer},
        Filter, Op,
    };

    /// equal =
    pub const EQ: Op = 1;
    /// not equal !=
    pub const NE: Op = 2;
    /// less than <
    pub const LT: Op = 3;
    /// less equal <=
    pub const LE: Op = 4;
    /// greater than >
    pub const GT: Op = 5;
    /// greater equal >=
    pub const GE: Op = 6;

    /// Equals [`Key`]
    pub fn eq<K>(key: K) -> Filter<K> {
        Filter(key, EQ)
    }

    /// Not Equals [`Key`]
    pub fn ne<K>(key: K) -> Filter<K> {
        Filter(key, NE)
    }

    /// Combine two [`Filter`] with an logical `OR`.
    pub fn or<'a, K, L: Indexer<K>, R: Indexer<K>>(
        lidx: &'a L,
        l: Filter<K>,
        ridx: &'a R,
        r: Filter<K>,
    ) -> Vec<&'a Idx> {
        let lr = lidx.index(l);
        let rr = ridx.index(r);

        let mut lhs: HashSet<&Idx> = HashSet::with_capacity(lr.len());
        lhs.extend(lr);
        let mut rhs = HashSet::with_capacity(rr.len());
        rhs.extend(rr);

        let r = &lhs | &rhs;
        r.iter().copied().collect()

        // Vec::<&Idx>::from_iter(lhs.symmetric_difference(&rhs).copied())
    }
}
