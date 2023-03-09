use std::{fmt::Debug, marker::PhantomData, ops::BitAnd};

use crate::{error::Error, Idx, Result};

#[allow(clippy::len_without_is_empty)]
pub trait Index: Debug {
    fn new(i: Idx) -> Self;
    fn add(&mut self, i: Idx) -> Result;
    fn get(&self) -> &[Idx];
    fn len(&self) -> usize;
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
        Err(Error::NotUniqueIndexKey)
    }

    #[inline]
    fn get(&self) -> &[Idx] {
        &self.0
    }

    #[inline]
    fn len(&self) -> usize {
        1
    }
}

impl BitAnd for &Unique {
    type Output = Option<Unique>;

    fn bitand(self, rhs: Self) -> Self::Output {
        let idx = self.0[0];
        if idx != rhs.0[0] {
            return None;
        }

        Some(Unique([idx]))
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Multi(Vec<Idx>);

impl Index for Multi {
    #[inline]
    fn new(i: Idx) -> Self {
        Multi(vec![i])
    }

    #[inline]
    fn add(&mut self, i: Idx) -> Result {
        match self.0.binary_search(&i) {
            Ok(_) => {} // i is already in vec
            Err(index) => self.0.insert(index, i),
        }
        Ok(())
    }

    #[inline]
    fn get(&self) -> &[Idx] {
        &self.0
    }

    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl BitAnd for &Multi {
    type Output = Option<Multi>;

    fn bitand(self, rhs: Self) -> Self::Output {
        let mut small = &self.0;
        let mut b = &rhs.0;

        if b.len() < small.len() {
            small = &rhs.0;
            b = &self.0;
        }

        let mut v = Vec::with_capacity(small.len());

        let len = b.len();
        let mut found = 0;

        for i in small.iter() {
            #[allow(clippy::needless_range_loop)]
            for j in found..len {
                if i == &b[j] {
                    v.push(*i);
                    found = j;
                    break;
                }
            }
        }

        Some(Multi(v))

        // let mut small = &self.0;
        // let mut b = &rhs.0;

        // if b.len() < small.len() {
        //     small = &rhs.0;
        //     b = &self.0;
        // }

        // let mut v = Vec::with_capacity(small.len());
        // for i in small.iter() {
        //     if b.contains(i) {
        //         v.push(*i);
        //     }
        // }

        // Some(Multi(v))
    }
}

/// Positions is an container for gathering [`Index`] values (&[Idx]).
/// It is usefull for operations like greater then ([`crate::Op::GT`]),
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

#[cfg(test)]
mod tests {

    mod multi {
        use super::super::*;

        #[test]
        fn and_1_to_10() {
            let m1 = Multi::new(1);
            let mut m2 = Multi::new(1);
            m2.add(0).unwrap();

            assert_eq!(m1.bitand(&m2).unwrap(), Multi::new(1));
        }

        #[test]
        fn and_10_to_1() {
            let mut m1 = Multi::new(1);
            m1.add(0).unwrap();
            let m2 = Multi::new(1);

            assert_eq!(m1.bitand(&m2).unwrap(), Multi::new(1));
        }

        #[test]
        fn and_many() {
            let mut m1 = Multi::new(1);
            m1.add(0).unwrap();
            m1.add(99).unwrap();

            let mut m2 = Multi::new(1);
            m2.add(99).unwrap();
            m2.add(200).unwrap();

            // 1 0 99 - 1 99 200 => 1 99
            assert_eq!(m1.bitand(&m2).unwrap(), Multi(vec![1, 99]));
        }
    }
}
