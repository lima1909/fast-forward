use std::{fmt::Debug, marker::PhantomData};

use crate::{error::Error, Idx, Result};

#[allow(clippy::len_without_is_empty)]
pub trait Index: Debug {
    fn new(i: Idx) -> Self;
    fn add(&mut self, i: Idx) -> Result;
    fn get(&self) -> &[Idx];
    fn len(&self) -> usize;
}

// Logical `And`, the intersection of two Inices.
pub trait And: Sized {
    fn and(&self, other: &[Idx]) -> Option<Self>;
}

// Logical `Or`, the union of two Inices.
pub trait Or {
    fn or(&self, other: &[Idx]) -> Multi;
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
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

impl And for Unique {
    fn and(&self, other: &[Idx]) -> Option<Self> {
        let idx = self.0[0];
        if other.contains(&idx) {
            return Some(Unique([idx]));
        }

        None
    }
}

impl Or for Unique {
    fn or(&self, other: &[Idx]) -> Multi {
        if other.is_empty() {
            return Multi(vec![self.0[0]]);
        }

        let mut v = Vec::from_iter(other.iter().copied());
        if !v.contains(&self.0[0]) {
            v.push(self.0[0])
        }
        Multi(v)
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
        if let Err(pos) = self.0.binary_search(&i) {
            self.0.insert(pos, i);
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

impl And for Multi {
    fn and(&self, other: &[Idx]) -> Option<Self> {
        let mut small = &self.0[..];
        let mut big = other;

        let mut ls = small.len();
        let mut lb = big.len();

        if lb < ls {
            small = other;
            big = &self.0;
            ls = other.len();
            lb = self.0.len();
        }

        let mut v = Vec::with_capacity(ls);
        let mut foundb = 0;

        for ss in small {
            #[allow(clippy::needless_range_loop)]
            for j in foundb..lb {
                let bb = big[j];

                #[allow(clippy::comparison_chain)]
                if ss < &bb {
                    break;
                } else if ss == &bb {
                    v.push(bb);
                    foundb += 1;
                    break;
                }
            }
        }

        Some(Multi(v))
    }
}

impl Or for Multi {
    fn or(&self, other: &[Idx]) -> Multi {
        match (self.0.is_empty(), other.is_empty()) {
            (false, false) => {
                use std::cmp::Ordering::*;

                let mut small = &self.0[..];
                let mut big = other;

                let mut ls = small.len();
                let mut lb = big.len();

                if lb < ls {
                    small = other;
                    big = &self.0;
                    ls = other.len();
                    lb = self.0.len();
                }

                let mut v = Vec::with_capacity(ls + lb);
                let mut foundb = 0;

                for ss in small {
                    if foundb == lb {
                        v.push(*ss);
                    }

                    #[allow(clippy::needless_range_loop)]
                    #[allow(clippy::mut_range_bound)]
                    for j in foundb..lb {
                        let bb = big[j];

                        match ss.cmp(&bb) {
                            Equal => {
                                v.push(bb);
                                foundb += 1;
                                break;
                            }
                            Less => {
                                v.push(*ss);
                                break;
                            }
                            Greater => {
                                v.push(bb);
                                foundb += 1;
                            }
                        }
                    }
                }

                if foundb < lb {
                    v.extend(big[foundb..].iter());
                }

                Multi(v)
            }
            (true, false) => return Multi(Vec::from_iter(other.iter().copied())),
            (false, true) => return Multi(Vec::from_iter(self.0.iter().copied())),
            // should be impossible
            (true, true) => unreachable!("Not valid OR state, self and other are empty"),
        }
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

    mod overlapping {
        use super::super::*;

        #[test]
        fn overlapping() {
            let mut lm = Multi::new(0);
            let mut lv = Vec::new();
            for i in 0..50 {
                lv.push(i);
                if i > 0 {
                    lm.add(i).unwrap();
                }
            }

            let mut rm = Multi::new(25);
            let mut rv = Vec::new();
            for i in 25..75 {
                rv.push(i);
                if i > 25 {
                    rm.add(i).unwrap();
                }
            }

            assert_eq!(25, lm.and(rm.get()).unwrap().len());
            assert_eq!(25, rm.and(lm.get()).unwrap().len());

            assert_eq!(75, lm.or(rm.get()).len());
            assert_eq!(75, rm.or(lm.get()).len());
        }
    }

    mod or {
        use super::super::*;

        #[test]
        fn or_1_to_10() {
            let m1 = Multi::new(1);
            let mut m2 = Multi::new(1);
            m2.add(0).unwrap();

            // 1 - 0 1 => 0 1
            assert_eq!(m1.or(m2.get()).get(), &vec![0, 1]);
            assert_eq!(m2.or(m1.get()).get(), &vec![0, 1]);
        }

        #[test]
        fn or_10_to_1() {
            let mut m1 = Multi::new(1);
            m1.add(0).unwrap();
            let m2 = Multi::new(1);

            // 0 1 - 1 => 0 1
            assert_eq!(m1.or(m2.get()).get(), &vec![0, 1]);
            assert_eq!(m2.or(m1.get()).get(), &vec![0, 1]);
        }

        #[test]
        fn or_unique_5_to_1_99() {
            let u = Unique::new(5);
            let mut m = Multi::new(99);
            m.add(1).unwrap();

            // 5 - 1 99 => 1 5 99
            assert_eq!(m.or(u.get()).get(), &vec![1, 5, 99]);
        }
    }

    mod and {
        use super::super::*;

        #[test]
        fn and_1_to_10() {
            let m1 = Multi::new(1);
            let mut m2 = Multi::new(1);
            m2.add(0).unwrap();

            assert_eq!(m1.and(m2.get()).unwrap(), Multi::new(1));
        }

        #[test]
        fn and_10_to_1() {
            let mut m1 = Multi::new(1);
            m1.add(0).unwrap();
            let m2 = Multi::new(1);

            assert_eq!(m1.and(m2.get()).unwrap(), Multi::new(1));
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
            assert_eq!(m1.and(m2.get()).unwrap(), Multi(vec![1, 99]));
        }

        #[test]
        fn and_many_duplicate() {
            let mut m1 = Multi::new(1);
            m1.add(99).unwrap();
            m1.add(0).unwrap();
            m1.add(99).unwrap();

            let mut m2 = Multi::new(1);
            m2.add(99).unwrap();
            m2.add(200).unwrap();
            m2.add(1).unwrap();

            // 1 (99) 0 99 - 1 99 200 (1) => 1 99
            assert_eq!(m1.and(m2.get()).unwrap(), Multi(vec![1, 99]));
        }

        #[test]
        fn and_many_and_unique() {
            let mut m1 = Multi::new(1);
            m1.add(0).unwrap();
            m1.add(99).unwrap();

            let mut m2 = Multi::new(1);
            m2.add(99).unwrap();
            m2.add(200).unwrap();

            // 1 0 99 - 1 99 200 - 99 => 99
            assert_eq!(
                m1.and(m2.get())
                    .unwrap()
                    .and(Unique::new(99).get())
                    .unwrap(),
                Multi(vec![99])
            );
        }

        #[test]
        fn and_unique_and_many() {
            let mut m1 = Multi::new(1);
            m1.add(0).unwrap();
            m1.add(99).unwrap();

            let mut m2 = Multi::new(1);
            m2.add(99).unwrap();
            m2.add(200).unwrap();

            // 99 - 1 99 200 - 1 0 99 => 99
            assert_eq!(
                Unique::new(99)
                    .and(m2.get())
                    .unwrap()
                    .and(m1.get())
                    .unwrap(),
                Unique([99])
            );
        }
    }
}
