//! Query combines different filter. Filters can be linked using `and` and `or`.
use crate::{index::Filterable, Idx, Predicate};
use std::{
    borrow::Cow,
    collections::HashSet,
    marker::PhantomData,
    ops::{BitAnd, BitOr},
};

pub trait Queryable<'k> {
    /// `pk` (name) `=` (ops::EQ) `6` (Key::Usize(6))
    fn filter<P>(&self, p: P) -> Cow<[usize]>
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
    fn filter<P>(&self, p: P) -> Cow<[usize]>
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

impl<'k, 'q, Q, B> QueryBuilder<'q, Q, B>
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
        Query {
            q: self.q,
            ors: Ors::new(idxs),
        }
    }
}

/// Query combines different filter. Filters can be linked using `and` and `or`.
pub struct Query<'q, Q, B: BinOp = HashSet<Idx>> {
    q: &'q Q,
    ors: Ors<'q, B>,
}

impl<'k, 'q, Q, B> Query<'q, Q, B>
where
    Q: Queryable<'k>,
    B: BinOp,
{
    pub fn or<P>(mut self, p: P) -> Self
    where
        P: Into<Predicate<'k>>,
    {
        let idxs = self.q.filter(p.into());
        self.ors.or(B::from_idx(&idxs));
        self
    }

    pub fn and<P>(mut self, p: P) -> Self
    where
        P: Into<Predicate<'k>>,
    {
        let idxs = self.q.filter(p.into());
        self.ors.and(B::from_idx(&idxs));
        self
    }

    pub fn exec(self) -> Iter<'q> {
        self.ors.exec()
    }
}

struct Ors<'s, B: BinOp = HashSet<Idx>> {
    idxs: Cow<'s, [usize]>, // lazy, convert to first<B>, only if added more B's with and/or
    first: Option<B>,
    ors: Vec<B>,
}

impl<'s, B: BinOp> Ors<'s, B> {
    const fn new(idxs: Cow<'s, [usize]>) -> Self {
        Self {
            idxs,
            first: None,
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
            let first = match &mut self.first {
                Some(first) => first,
                None => {
                    self.first = Some(B::from_idx(&self.idxs));
                    self.first.as_mut().unwrap()
                }
            };
            self.first = Some(first.and(&b));
        } else {
            let i = self.ors.len() - 1;
            self.ors[i] = self.ors[i].and(&b);
        }
    }

    #[inline]
    fn exec(mut self) -> Iter<'s> {
        if self.ors.is_empty() {
            return match self.first.take() {
                Some(first) => first.iter(),
                // None => Iter::Slice((self.idxs).iter().collect()), TODO !!!
                _ => B::from_idx(&self.idxs).iter(),
            };
        }

        let mut first = match self.first {
            Some(first) => first,
            None => B::from_idx(&self.idxs),
        };

        // TODO: maybe it is better a sorted Vec by B.len() before executed???
        for b in self.ors {
            first = first.or(&b);
        }
        first.iter()
    }
}

pub enum Iter<'a> {
    Roaring(roaring::bitmap::IntoIter),
    HashSet(std::collections::hash_set::IntoIter<Idx>),
    Slice(std::slice::Iter<'a, Idx>),
}

impl<'a> Iterator for Iter<'a> {
    type Item = Idx;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Iter::Slice(it) => it.next().copied(),
            Iter::Roaring(it) => it.next().map(|u| u as usize),
            Iter::HashSet(it) => it.next(),
        }
    }
}

/// Support for binary logical operations, like `or` and `and`.
pub trait BinOp {
    //} <'a>: IntoIterator<Item = Idx, IntoIter = Iter<'a>> {
    fn from_idx(idx: &[Idx]) -> Self;
    fn iter<'a>(self) -> Iter<'a>;

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
    fn iter<'a>(self) -> Iter<'a> {
        Iter::HashSet(self.into_iter())
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
    fn iter<'a>(self) -> Iter<'a> {
        Iter::Roaring(self.into_iter())
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
