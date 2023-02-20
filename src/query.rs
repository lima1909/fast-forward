#![allow(dead_code)]

use crate::{
    index::{uint::UIntVecIndex, Unique},
    Filter, Idx, IdxFilter, Op,
};
use std::{
    collections::HashSet,
    marker::PhantomData,
    ops::{BitAnd, BitOr},
};

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

// pk (name) = (ops::EQ) 6 (Key::Usize(6))
pub struct QFilter<'a> {
    name: &'a str,
    op: Op,
    key: Key<'a>,
}

impl<'a, K: From<Key<'a>>> From<QFilter<'a>> for Filter<K> {
    fn from(f: QFilter<'a>) -> Self {
        Filter {
            op: f.op,
            key: f.key.into(),
        }
    }
}

impl<'a> QFilter<'a> {
    pub fn new(op: Op, key: Key<'a>) -> Self {
        Self { name: "", op, key }
    }
}

// pub trait FilterQuery {
//     fn query<'a>(f: QFilter<'a>) -> &[Idx];
// }

// needs:
// - one or many IdxFilter
// - FromIdx impl
pub trait NQuery<'a> {
    fn filter(self, f: QFilter<'a>) -> Self;
    fn or(self, f: QFilter<'a>) -> Self;
    fn exec(&self) -> Vec<Idx>;
}

pub struct OneIdxFilterQuery<F: FromIdx, K, I: IdxFilter<K>> {
    idx_filter: I,
    indices: F,
    _key: PhantomData<K>,
}

impl OneIdxFilterQuery<HashSet<Idx>, usize, UIntVecIndex<Unique>> {
    pub fn new(idx_filter: UIntVecIndex<Unique>) -> Self {
        Self {
            idx_filter,
            indices: HashSet::<Idx>::default(),
            _key: PhantomData,
        }
    }
}

impl<'a> NQuery<'a> for OneIdxFilterQuery<HashSet<Idx>, usize, UIntVecIndex<Unique>> {
    fn filter(mut self, f: QFilter<'a>) -> Self {
        let idxs = self.idx_filter.idx(f.into());
        self.indices = HashSet::<Idx>::from_idx(idxs);
        self
    }

    fn or(mut self, f: QFilter<'a>) -> Self {
        let idxs = self.idx_filter.idx(f.into());
        self.indices = HashSet::<Idx>::from_idx(idxs).bitor(&self.indices);
        self
    }

    fn exec(&self) -> Vec<Idx> {
        self.indices.iter().copied().collect()
    }
}

pub trait FromIdx {
    fn from_idx(idx: &[Idx]) -> Self;
}

impl FromIdx for HashSet<Idx> {
    fn from_idx(idx: &[Idx]) -> Self {
        let mut hs = HashSet::with_capacity(idx.len());
        hs.extend(idx);
        hs
    }
}

pub trait BinaryOperator: Sized {
    // fn from_idx(idx: &[Idx]) -> Self;
    fn to_idx(&self) -> Vec<Idx>;

    fn and(&self, rhs: &[Idx]) -> Self;
    // fn and<A: BitAnd>(&self, rhs: &[Idx]) -> Self {
    //     let rhs = Self::from_idx(rhs);
    //     self.bitand(&rhs)
    // }
}

impl BinaryOperator for HashSet<Idx> {
    // fn from_idx(idx: &[Idx]) -> Self {
    //     let mut hs = HashSet::with_capacity(idx.len());
    //     hs.extend(idx);
    //     hs
    // }

    fn to_idx(&self) -> Vec<Idx> {
        self.iter().copied().collect()
    }

    fn and(&self, rhs: &[Idx]) -> Self {
        let rhs = Self::from_idx(rhs);
        self.bitand(&rhs)
    }
}

#[cfg(feature = "roaring")]
impl BinaryOperator for roaring::RoaringBitmap {
    // fn from_idx(idx: &[Idx]) -> Self {
    //     idx.iter().map(|i| *i as u32).collect()
    // }

    fn to_idx(&self) -> Vec<Idx> {
        self.iter().map(|i| i as usize).collect()
    }

    fn and(&self, _rhs: &[Idx]) -> Self {
        // let bit: roaring::RoaringBitmap = Self::from_idx(rhs);
        // self.bitand(&bit)
        todo!()
    }
}
