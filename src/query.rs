//! Query combines different filter. Filters can be linked using `and` and `or`.
use crate::{
    index::{self},
    Idx, Op,
};
use std::{
    collections::{HashSet, VecDeque},
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

pub trait IdxFilter<'a> {
    fn filter(&'a self, f: Filter<'a>) -> &[Idx];
}

impl<'a, F: Fn(Filter<'a>) -> &[Idx]> IdxFilter<'a> for F {
    fn filter(&'a self, f: Filter<'a>) -> &[Idx] {
        self(f)
    }
}

pub struct QueryBuilder<B, I> {
    idx: I,
    _b: PhantomData<B>,
}

impl<'a, B, I> QueryBuilder<B, I>
where
    B: BinOp,
    I: IdxFilter<'a>,
{
    pub fn new(idx: I) -> Self {
        Self {
            idx,
            _b: PhantomData,
        }
    }

    pub fn query(&'a self, f: Filter<'a>) -> Query<B, I> {
        let idxs = self.idx.filter(f);
        let ors = Ors::new(B::from_idx(idxs));
        Query {
            idx: &self.idx,
            ors,
        }
    }
}

/// Query combines different filter. Filters can be linked using `and` and `or`.
pub struct Query<'a, B, I> {
    idx: &'a I,
    ors: Ors<B>,
}

impl<'a, B, I> Query<'a, B, I>
where
    B: BinOp,
    I: IdxFilter<'a>,
{
    pub fn or(mut self, f: Filter<'a>) -> Self {
        let idxs = self.idx.filter(f);
        self.ors.or(B::from_idx(idxs));
        self
    }

    pub fn and(mut self, f: Filter<'a>) -> Self {
        let idxs = self.idx.filter(f);
        self.ors.and(B::from_idx(idxs));
        self
    }

    pub fn exec(self) -> Vec<Idx> {
        self.ors.exec()
    }
}

struct Ors<B>(VecDeque<B>);

impl<B: BinOp> Ors<B> {
    fn new(b: B) -> Self {
        Self(VecDeque::from(vec![b]))
    }

    #[inline]
    fn or(&mut self, b: B) {
        self.0.push_back(b);
    }

    #[inline]
    fn and(&mut self, b: B) {
        let i = self.0.len() - 1;
        self.0[i] = self.0[i].and(&b);
    }

    #[inline]
    fn exec(mut self) -> Vec<Idx> {
        let mut first = self.0.pop_front().unwrap();
        for b in self.0 {
            first = first.or(&b);
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
