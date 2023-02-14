use std::ops::Index;

use index::{Idx, Key};

pub mod index;

/// Id for operations. The default operations are [`DefaultOp`]
pub type Op = u8;

/// Filter is a given query key: [`Key`] and operation: [`Op`]
pub struct Filter(Key, Op);

impl Filter {
    #[inline]
    pub fn key(&self) -> &Key {
        &self.0
    }

    #[inline]
    pub fn op(&self) -> Op {
        self.1
    }
}

pub trait Query: Index<Filter, Output = [Idx]> + Sized {
    fn filter(&self, f: Filter) -> &[Idx] {
        &self[f]
    }

    fn or_rhs<'a, Rhs: Index<Filter, Output = [Idx]>>(
        &'a self,
        l: Filter,
        ridx: &'a Rhs,
        r: Filter,
    ) -> Vec<&'a Idx> {
        ops::or(self, l, ridx, r)
    }

    fn or(&self, l: Filter, r: Filter) -> Vec<&Idx> {
        ops::or::<Self, Self>(self, l, self, r)
    }
}

impl<I: Index<Filter, Output = [Idx]> + Sized> Query for I {}

pub mod ops {
    use std::{collections::HashSet, ops::Index};

    use crate::{
        index::{Idx, Key},
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
    pub fn eq<K: Into<Key>>(key: K) -> Filter {
        Filter(key.into(), EQ)
    }

    /// Not Equals [`Key`]
    pub fn ne<K: Into<Key>>(key: K) -> Filter {
        Filter(key.into(), NE)
    }

    /// Combine two [`Filter`] with an logical `OR`.
    pub fn or<'a, L, R>(lidx: &'a L, l: Filter, ridx: &'a R, r: Filter) -> Vec<&'a Idx>
    where
        L: Index<Filter, Output = [Idx]>,
        R: Index<Filter, Output = [Idx]>,
    {
        let lr = &lidx[l];
        let rr = &ridx[r];

        let mut lhs: HashSet<&Idx> = HashSet::with_capacity(lr.len());
        lhs.extend(lr);
        let mut rhs = HashSet::with_capacity(rr.len());
        rhs.extend(rr);

        let r = &lhs | &rhs;
        r.iter().copied().collect()

        // Vec::<&Idx>::from_iter(lhs.symmetric_difference(&rhs).copied())
    }
}
