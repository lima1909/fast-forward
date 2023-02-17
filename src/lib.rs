pub mod index;
pub mod ops;

/// `Idx` is the index/position in a List ([`std::vec::Vec`]).
pub type Idx = usize;

/// Id for operations.
/// Operations are primarily compare functions, like equal, greater than and so on.
pub type Op = u8;

/// Filter are the input data for describung a filter. A filter consist of a key and a operation [`Op`].
/// Key `K` is a unique value under which all occurring indices are stored.
///
/// For example:
/// Filter `= 5`
/// means: Op: `=` and Key: `5`
pub struct Filter<K> {
    pub op: Op,
    pub key: K,
}

impl<K> Filter<K> {
    fn new(op: Op, key: K) -> Self {
        Self { op, key }
    }
}
/// Find all [`Idx`] for an given [`crate::Op`] and `Key`.
pub trait IdxFilter<K> {
    fn idx(&self, f: Filter<K>) -> &[Idx];
}

/// Query combines different filter. Filters can be linked using `and` and `or`.
pub trait Query<K>: IdxFilter<K> + Sized {
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

impl<K, I: IdxFilter<K> + Sized> Query<K> for I {}
