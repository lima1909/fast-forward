//! Operations are primarily compare functions, like equal, greater than and so on.
use std::collections::HashSet;

use crate::{Filter, Idx, IdxFilter, Op};

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

/// Equals `Key`
pub fn eq<K>(key: K) -> Filter<K> {
    Filter::new(EQ, key)
}

/// Not Equals `Key`
pub fn ne<K>(key: K) -> Filter<K> {
    Filter::new(NE, key)
}

/// Combine two [`Filter`] with an logical `OR`.
pub fn or<'a, K, L: IdxFilter<K>, R: IdxFilter<K>>(
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
