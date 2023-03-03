//! Query combines different filter. Filters can be linked using `and` and `or`.
use crate::{
    index::{self},
    ops::EQ,
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
    pub field: &'a str,
    pub op: Op,
    pub key: Key<'a>,
}

impl<'a> Filter<'a> {
    pub fn new(field: &'a str, op: Op, key: Key<'a>) -> Self {
        Self { field, op, key }
    }
}

impl<'a, K> From<Filter<'a>> for index::Filter<K>
where
    K: From<Key<'a>>,
{
    fn from(f: Filter<'a>) -> Self {
        index::Filter {
            op: f.op,
            key: f.key.into(),
        }
    }
}

impl<'a, K> From<(Op, K)> for Filter<'a>
where
    K: Into<Key<'a>>,
{
    fn from(ok: (Op, K)) -> Self {
        Filter::new("", ok.0, ok.1.into())
    }
}

impl<'a, K> From<K> for Filter<'a>
where
    K: Into<Key<'a>>,
{
    fn from(k: K) -> Self {
        Filter::new("", EQ, k.into())
    }
}

pub trait IdxFilter<'f> {
    fn filter(&self, f: Filter<'f>) -> &[Idx];
}

pub trait IdxFilterQuery<'f>: IdxFilter<'f> + Sized {
    fn query_builder<B: BinOp>(&self) -> QueryBuilder<Self, B> {
        QueryBuilder::<_, B>::new(self)
    }
}

pub struct QueryBuilder<'i, I, B: BinOp = HashSet<Idx>> {
    idx: &'i I,
    _b: PhantomData<B>,
}

impl<'i, 'a, I, B> QueryBuilder<'i, I, B>
where
    I: IdxFilter<'a>,
    B: BinOp,
{
    pub fn new(idx: &'i I) -> Self {
        Self {
            idx,
            _b: PhantomData,
        }
    }

    pub fn query<F>(&self, f: F) -> Query<I, B>
    where
        F: Into<Filter<'a>>,
    {
        let idxs = self.idx.filter(f.into());
        let ors = Ors::new(B::from_idx(idxs));
        Query { idx: self.idx, ors }
    }
}

/// Query combines different filter. Filters can be linked using `and` and `or`.
pub struct Query<'i, I, B: BinOp = HashSet<Idx>> {
    idx: &'i I,
    ors: Ors<B>,
}

impl<'i, 'f, I, B> Query<'i, I, B>
where
    I: IdxFilter<'f> + 'i,
    B: BinOp,
{
    pub fn or<F>(mut self, f: F) -> Self
    where
        F: Into<Filter<'f>>,
    {
        let idxs = self.idx.filter(f.into());
        self.ors.or(B::from_idx(idxs));
        self
    }

    pub fn and<F>(mut self, f: F) -> Self
    where
        F: Into<Filter<'f>>,
    {
        let idxs = self.idx.filter(f.into());
        self.ors.and(B::from_idx(idxs));
        self
    }

    pub fn exec(self) -> Vec<Idx> {
        self.ors.exec()
    }
}

struct Ors<B: BinOp = HashSet<Idx>> {
    first: B,
    ors: Vec<B>,
}

impl<B: BinOp> Ors<B> {
    fn new(b: B) -> Self {
        Self {
            first: b,
            ors: vec![],
        }
    }

    #[inline]
    fn or(&mut self, b: B) {
        self.ors.push(b);
    }

    #[inline]
    fn and(&mut self, b: B) {
        if self.ors.is_empty() {
            self.first = self.first.and(&b);
        } else {
            let i = self.ors.len() - 1;
            self.ors[i] = self.ors[i].and(&b);
        }
    }

    #[inline]
    fn exec(mut self) -> Vec<Idx> {
        // TODO: maybe it is better a sorted Vec by B.len() before executed???
        for b in self.ors {
            self.first = self.first.or(&b);
        }
        self.first.to_idx()
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
