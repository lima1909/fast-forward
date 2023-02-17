pub mod index;

/// `Key` is a unique value under which all occurring indices are stored.
pub trait Key {}

impl Key for usize {}
impl Key for &str {}

/// `Idx` is the index/position in a List ([`std::vec::Vec`]).
pub type Idx = usize;

/// Id for operations.
/// Operations are primarily compare functions, like equal, greater than and so on.
pub type Op = u8;

/// Filter are the input data for describung a filter.
///
/// For example:
/// Filter `= 5`
/// means: Op: `=` and Key: `5`
pub struct Filter<K>(Op, K);

impl<K> Filter<K> {
    #[inline]
    pub fn op(&self) -> Op {
        self.0
    }

    #[inline]
    pub fn key(&self) -> &K {
        &self.1
    }
}

/// Find all [`Idx`] for an given [`crate::Op`] and [`Key`].
pub trait IdxFilter<K: Key> {
    fn idx(&self, f: Filter<K>) -> &[Idx];
}

/// Query combines different filter. Filters can be linked using `and` and `or`.
pub trait Query<K: Key>: IdxFilter<K> + Sized {
    fn filter(&self, f: Filter<K>) -> &[Idx] {
        self.idx(f)
    }

    fn or_rhs<'a, Rhs: IdxFilter<K>>(
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

impl<K: Key, I: IdxFilter<K> + Sized> Query<K> for I {}

/// Operations are primarily compare functions, like equal, greater than and so on.
pub mod ops {
    use std::collections::HashSet;

    use crate::{Filter, Idx, IdxFilter, Key, Op};

    /// equal `=`
    pub const EQ: Op = 1;
    /// not equal `!=`
    pub const NE: Op = 2;
    /// less than `<`
    pub const LT: Op = 3;
    /// less equal `<=`
    pub const LE: Op = 4;
    /// greater than `>`
    pub const GT: Op = 5;
    /// greater equal `>=`
    pub const GE: Op = 6;

    /// Equals [`Key`]
    pub fn eq<K>(key: K) -> Filter<K> {
        Filter(EQ, key)
    }

    /// Not Equals [`Key`]
    pub fn ne<K>(key: K) -> Filter<K> {
        Filter(NE, key)
    }

    /// Combine two [`Filter`] with an logical `OR`.
    pub fn or<'a, K: Key, L: IdxFilter<K>, R: IdxFilter<K>>(
        lidx: &'a L,
        l: Filter<K>,
        ridx: &'a R,
        r: Filter<K>,
    ) -> Vec<&'a Idx> {
        let lr = lidx.idx(l);
        let rr = ridx.idx(r);

        let mut lhs: HashSet<&Idx> = HashSet::with_capacity(lr.len());
        lhs.extend(lr);
        let mut rhs = HashSet::with_capacity(rr.len());
        rhs.extend(rr);

        let r = &lhs | &rhs;
        r.iter().copied().collect()

        // Vec::<&Idx>::from_iter(lhs.symmetric_difference(&rhs).copied())
    }
}
