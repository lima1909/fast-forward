//! Query combines different filter. Filters can be linked using `and` and `or`.
use crate::{
    index::{self, IdxFilter},
    Idx, Op,
};
use std::{collections::HashSet, marker::PhantomData, ops::BitOr};

/// Supported types for quering [`IdxFilter`].
#[derive(Debug, Clone)]
pub enum Key<'a> {
    Usize(usize),
    Str(&'a str),
}

impl<'a> From<Key<'a>> for usize {
    fn from(key: Key<'a>) -> Self {
        match key {
            Key::Usize(u) => u,
            _ => todo!(),
        }
    }
}

impl<'a> From<Key<'a>> for &'a str {
    fn from(key: Key<'a>) -> Self {
        match key {
            Key::Str(s) => s,
            _ => todo!(),
        }
    }
}

impl From<usize> for Key<'_> {
    fn from(u: usize) -> Self {
        Key::Usize(u)
    }
}

impl<'a> From<&'a str> for Key<'a> {
    fn from(s: &'a str) -> Self {
        Key::Str(s)
    }
}
/// `pk` (name) `=` (ops::EQ) `6` (Key::Usize(6))
pub struct Filter<'a> {
    #[allow(dead_code)]
    field: &'a str,
    op: Op,
    key: Key<'a>,
}

impl<'a> Filter<'a> {
    pub fn new(field: &'a str, op: Op, key: Key<'a>) -> Self {
        Self { field, op, key }
    }
}

impl<'a, K: From<Key<'a>>> From<Filter<'a>> for index::Filter<K> {
    fn from(f: Filter<'a>) -> Self {
        index::Filter {
            op: f.op,
            key: f.key.into(),
        }
    }
}

// pub trait FilterQuery {
//     fn query<'a>(f: QFilter<'a>) -> &[Idx];
// }

/// Query combines different filter. Filters can be linked using `and` and `or`.
pub trait Query<'a> {
    fn filter(&mut self, f: Filter<'a>) -> &mut Self;
    fn or(&mut self, f: Filter<'a>) -> &mut Self;
    fn reset(&mut self) -> &mut Self;
    fn exec(&self) -> Vec<Idx>;
}

/// If this trait is in scope, than it convert [`IdxFilter`] into a [`Query`].
pub trait ToQuery<B: BinOp, K>: IdxFilter<K> + Sized {
    fn to_query(self, bin_op: B) -> IdxFilterQuery<B, K, Self> {
        IdxFilterQuery::new(self, bin_op)
    }
}

impl<B: BinOp, K, I: IdxFilter<K>> ToQuery<B, K> for I {}

/// Wrapper, for creating an impl for the trait [`Query`] combined with the [`IdxFilter`] trait.
/// The simpelst way to use the [`ToQuery`] trait.
pub struct IdxFilterQuery<B, K, I> {
    idx_filter: I,
    indices: B,
    _key: PhantomData<K>,
}

impl<B, K, I> IdxFilterQuery<B, K, I> {
    pub fn new(idx_filter: I, bin_op: B) -> Self {
        Self {
            idx_filter,
            indices: bin_op,
            _key: PhantomData,
        }
    }
}

impl<'a, B, K, I> Query<'a> for IdxFilterQuery<B, K, I>
where
    B: BinOp,
    K: From<Key<'a>>,
    I: IdxFilter<K>,
{
    fn filter(&mut self, f: Filter<'a>) -> &mut Self {
        let idxs = self.idx_filter.idx(f.into());
        self.indices = B::from_idx(idxs);
        self
    }

    fn or(&mut self, f: Filter<'a>) -> &mut Self {
        let idxs = self.idx_filter.idx(f.into());
        let or = self.indices.or(idxs);
        let _old = std::mem::replace(&mut self.indices, or);
        self
    }

    fn exec(&self) -> Vec<Idx> {
        self.indices.to_idx()
    }

    fn reset(&mut self) -> &mut Self {
        self.indices.reset();
        self
    }
}

/// Support for binary logical operations, like `or` and `and`.
pub trait BinOp {
    fn from_idx(idx: &[Idx]) -> Self;
    fn to_idx(&self) -> Vec<Idx>;

    fn or(&self, idx: &[Idx]) -> Self;

    fn reset(&mut self);
}

impl BinOp for HashSet<Idx> {
    fn from_idx(idx: &[Idx]) -> Self {
        let mut hs = HashSet::with_capacity(idx.len());
        hs.extend(idx);
        hs
    }

    fn to_idx(&self) -> Vec<Idx> {
        self.iter().copied().collect()
    }

    fn or(&self, idx: &[Idx]) -> Self {
        let rhs = Self::from_idx(idx);
        self.bitor(&rhs)
    }

    fn reset(&mut self) {
        self.clear();
    }
}

#[cfg(feature = "roaring")]
impl BinOp for roaring::RoaringBitmap {
    fn from_idx(idx: &[Idx]) -> Self {
        idx.iter().map(|i| *i as u32).collect()
    }

    fn to_idx(&self) -> Vec<Idx> {
        self.iter().map(|i| i as usize).collect()
    }

    fn or(&self, idx: &[Idx]) -> Self {
        let rhs = Self::from_idx(idx);
        self.bitor(rhs)
    }

    fn reset(&mut self) {
        self.clear();
    }
}
