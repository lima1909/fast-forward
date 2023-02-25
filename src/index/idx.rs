use std::{fmt::Debug, marker::PhantomData};

use super::{IndexError, Result};
use crate::Idx;

pub trait Index: Debug {
    fn new(i: Idx) -> Self;
    fn add(&mut self, i: Idx) -> Result;
    fn get(&self) -> &[Idx];
}

#[derive(Debug, Default, Clone)]
pub struct Unique([Idx; 1]);

impl Index for Unique {
    #[inline]
    fn new(i: Idx) -> Self {
        Unique([i])
    }

    #[inline]
    fn add(&mut self, _i: Idx) -> Result {
        Err(IndexError::NotUniqueKey)
    }

    #[inline]
    fn get(&self) -> &[Idx] {
        &self.0
    }
}

#[derive(Debug, Default, Clone)]
pub struct Multi(Vec<Idx>);

impl Index for Multi {
    #[inline]
    fn new(i: Idx) -> Self {
        Multi(vec![i])
    }

    #[inline]
    fn add(&mut self, i: Idx) -> Result {
        self.0.push(i);
        Ok(())
    }

    #[inline]
    fn get(&self) -> &[Idx] {
        &self.0
    }
}

/// Positions is an container for gathering [`Index`] values (&[Idx]).
/// It is usefull for operations like greater then ([`crate::ops::GT`]),
/// where the result consists one or many [`Index`]s.
pub struct Positions<I>(Vec<Idx>, PhantomData<I>);

impl<I: Index> Positions<I> {
    #[inline]
    pub fn new(i: I) -> Self {
        Positions(Vec::from_iter(i.get().iter().copied()), PhantomData)
    }

    #[inline]
    pub fn add(&mut self, i: I) {
        self.0.extend(i.get());
    }

    #[inline]
    pub fn get(&self) -> &[Idx] {
        &self.0
    }
}
