//! Query combines different filter. Filters can be linked using `and` and `or`.
use crate::{index::Filterable, Idx, Predicate};
use std::{
    collections::HashSet,
    marker::PhantomData,
    ops::{BitAnd, BitOr},
};

pub trait Queryable<'k> {
    /// `pk` (name) `=` (ops::EQ) `6` (Key::Usize(6))
    fn filter<P>(&self, p: P) -> &[Idx]
    where
        P: Into<Predicate<'k>>;

    fn query_builder<B: BinOp>(&self) -> QueryBuilder<Self, B>
    where
        Self: Sized,
    {
        QueryBuilder::<_, B>::new(self)
    }
}

impl<'k, F: Filterable<'k>> Queryable<'k> for F {
    fn filter<P>(&self, p: P) -> &[Idx]
    where
        P: Into<Predicate<'k>>,
    {
        Filterable::filter(self, p.into())
    }
}

pub struct QueryBuilder<'q, Q, B: BinOp = HashSet<Idx>> {
    q: &'q Q,
    _b: PhantomData<B>,
}

impl<'q, 'k, Q, B> QueryBuilder<'q, Q, B>
where
    Q: Queryable<'k>,
    B: BinOp,
{
    pub const fn new(q: &'q Q) -> Self {
        Self { q, _b: PhantomData }
    }

    pub fn query<P>(&self, p: P) -> Query<Q, B>
    where
        P: Into<Predicate<'k>>,
    {
        let idxs = self.q.filter(p.into());
        let ors = Ors::new(B::from_idx(idxs));
        Query { q: self.q, ors }
    }
}

/// Query combines different filter. Filters can be linked using `and` and `or`.
pub struct Query<'q, Q, B: BinOp = HashSet<Idx>> {
    q: &'q Q,
    ors: Ors<B>,
}

impl<'q, 'k, Q, B> Query<'q, Q, B>
where
    Q: Queryable<'k> + 'q,
    B: BinOp,
{
    pub fn or<P>(mut self, p: P) -> Self
    where
        P: Into<Predicate<'k>>,
    {
        let idxs = self.q.filter(p.into());
        self.ors.or(B::from_idx(idxs));
        self
    }

    pub fn and<P>(mut self, p: P) -> Self
    where
        P: Into<Predicate<'k>>,
    {
        let idxs = self.q.filter(p.into());
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
    const fn new(b: B) -> Self {
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
