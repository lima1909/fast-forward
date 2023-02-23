//! Query combines different filter. Filters can be linked using `and` and `or`.
use crate::{
    index::{self, IdxFilter},
    Idx, Op,
};
use std::{
    collections::HashSet,
    marker::PhantomData,
    ops::{BitAnd, BitOr},
};

/// Supported types for quering/filtering [`IdxFilter`].
#[derive(Debug, Clone)]
pub enum Key<'a> {
    Usize(usize),
    Str(&'a str),
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

/// Query combines different filter. Filters can be linked using `and` and `or`.
pub trait Query<'a> {
    #[allow(clippy::wrong_self_convention)]
    fn new(&mut self, f: Filter<'a>) -> &mut Self;
    fn or(&mut self, f: Filter<'a>) -> &mut Self;
    fn and(&mut self, f: Filter<'a>) -> &mut Self;
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
    ors: Ors<B>,
    _key: PhantomData<K>,
}

impl<B: BinOp, K, I> IdxFilterQuery<B, K, I> {
    pub fn new(idx_filter: I, bin_op: B) -> Self {
        Self {
            idx_filter,
            ors: Ors::new(bin_op),
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
    fn new(&mut self, f: Filter<'a>) -> &mut Self {
        let idxs = self.idx_filter.idx(f.into());
        self.ors = Ors::new(B::from_idx(idxs));
        self
    }

    fn or(&mut self, f: Filter<'a>) -> &mut Self {
        let idxs = self.idx_filter.idx(f.into());
        self.ors.or(B::from_idx(idxs));
        self
    }

    fn and(&mut self, f: Filter<'a>) -> &mut Self {
        let idxs = self.idx_filter.idx(f.into());
        self.ors.and(B::from_idx(idxs));
        self
    }

    fn exec(&self) -> Vec<Idx> {
        self.ors.exec()
    }
}

struct Ors<B> {
    current_pos: usize,
    first: B, // equals ands
    list: Vec<B>,
}

impl<B: BinOp> Ors<B> {
    fn new(b: B) -> Self {
        Self {
            current_pos: 0,
            first: b,
            list: Vec::new(),
        }
    }

    // or is equals add and exec on the end with call `exec`
    fn or(&mut self, b: B) {
        self.list.push(b);
        self.current_pos += 1;
    }

    fn and(&mut self, b: B) {
        if self.current_pos == 0 {
            self.first = self.first.and(&b)
        } else {
            let i = self.current_pos - 1;
            self.list[i] = self.list[i].and(&b);
        }
    }

    #[inline]
    fn exec(&self) -> Vec<Idx> {
        if self.list.is_empty() {
            return self.first.to_idx();
        }

        let mut first = self.first.or(&self.list[0]);
        for b in self.list.iter().skip(1) {
            first = first.or(b)
        }
        first.to_idx()
    }
}

/// Support for binary logical operations, like `or` and `and`.
pub trait BinOp {
    fn from_idx(idx: &[Idx]) -> Self;
    fn to_idx(&self) -> Vec<Idx>;

    fn or(&self, idx: &Self) -> Self;
    fn and(&self, idx: &Self) -> Self;
}

impl BinOp for HashSet<Idx> {
    fn from_idx(idx: &[Idx]) -> Self {
        let mut hs = HashSet::with_capacity(idx.len());
        hs.extend(idx);
        hs
    }

    #[inline]
    fn to_idx(&self) -> Vec<Idx> {
        self.iter().copied().collect()
    }

    #[inline]
    fn or(&self, idx: &Self) -> Self {
        self.bitor(idx)
    }

    #[inline]
    fn and(&self, idx: &Self) -> Self {
        self.bitand(idx)
    }
}

#[cfg(feature = "roaring")]
impl BinOp for roaring::RoaringBitmap {
    fn from_idx(idx: &[Idx]) -> Self {
        idx.iter().map(|i| *i as u32).collect()
    }

    #[inline]
    fn to_idx(&self) -> Vec<Idx> {
        self.iter().map(|i| i as usize).collect()
    }

    #[inline]
    fn or(&self, idx: &Self) -> Self {
        self.bitor(idx)
    }

    #[inline]
    fn and(&self, idx: &Self) -> Self {
        self.bitand(idx)
    }
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
