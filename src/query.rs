//! Query combines different filter. Filters can be linked using `and` and `or`.
use crate::{
    index::{self},
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

impl<'a, B: BinOp, I: IdxFilter<'a>> QueryBuilder<B, I> {
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

impl<'a, B: BinOp, I: IdxFilter<'a>> Query<'a, B, I> {
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

    pub fn exec(mut self) -> Vec<Idx> {
        self.ors.exec()
    }
}

struct Ors<B> {
    ops: Vec<B>,
}

impl<B: BinOp> Ors<B> {
    fn new(b: B) -> Self {
        Self { ops: vec![b] }
    }

    #[inline]
    fn or(&mut self, b: B) {
        self.ops.push(b);
    }

    #[inline]
    fn and(&mut self, b: B) {
        let i = self.ops.len() - 1;
        self.ops[i] = self.ops[i].and(&b);
    }

    #[inline]
    fn exec(&mut self) -> Vec<Idx> {
        let v = std::mem::take(&mut self.ops);
        let mut it = v.into_iter();
        let mut first = it.next().unwrap();
        for b in it {
            first = first.or(&b);
        }

        let idx = first.to_idx();
        self.or(first);
        idx
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
