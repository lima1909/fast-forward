//! Operation module, e.g. [`union`] or [`intersection`].
use std::{
    borrow::Cow,
    cmp::{min, Ordering::*},
};

/// Union is using for OR
#[inline]
pub fn union<'a, I: Ord + Clone>(lhs: Cow<'a, [I]>, rhs: Cow<'a, [I]>) -> Cow<'a, [I]> {
    if lhs.is_empty() {
        return rhs;
    }
    if rhs.is_empty() {
        return lhs;
    }

    let (ll, lr) = (lhs.len(), rhs.len());
    let mut v = Vec::with_capacity(ll + lr);

    let (mut li, mut ri) = (0, 0);

    loop {
        let (l, r) = (lhs[li].clone(), rhs[ri].clone());

        match l.cmp(&r) {
            Equal => {
                v.push(l);
                li += 1;
                ri += 1;
            }
            Less => {
                v.push(l);
                li += 1;
            }
            Greater => {
                v.push(r);
                ri += 1;
            }
        }

        if ll == li {
            v.extend(rhs.iter().skip(ri).cloned());
            return Cow::Owned(v);
        } else if lr == ri {
            v.extend(lhs.iter().skip(li).cloned());
            return Cow::Owned(v);
        }
    }
}

/// Intersection is using for AND
#[inline]
pub fn intersection<'a, I: Ord + Clone>(lhs: Cow<'a, [I]>, rhs: Cow<'a, [I]>) -> Cow<'a, [I]> {
    if lhs.is_empty() {
        return lhs;
    }
    if rhs.is_empty() {
        return rhs;
    }

    let (ll, lr) = (lhs.len(), rhs.len());
    let mut v = Vec::with_capacity(min(ll, lr));

    let (mut li, mut ri) = (0, 0);

    loop {
        let l = lhs[li].clone();

        match l.cmp(&rhs[ri]) {
            Equal => {
                v.push(l);
                li += 1;
                ri += 1;
            }
            Less => li += 1,
            Greater => ri += 1,
        }

        if li == ll || ri == lr {
            return Cow::Owned(v);
        }
    }
}

#[derive(Debug, Default)]
pub struct MinMax<K> {
    pub min: K,
    pub max: K,
}

impl<K: Default + Ord> MinMax<K> {
    pub fn new_min_value(&mut self, key: K) -> &K {
        if self.min == K::default() || self.min > key {
            self.min = key
        }
        &self.min
    }

    pub fn new_max_value(&mut self, key: K) -> &K {
        if self.max < key {
            self.max = key
        }
        &self.max
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod min_max {
        use super::*;

        #[test]
        fn min() {
            assert_eq!(0, MinMax::default().min);
            assert_eq!(&0, MinMax::default().new_min_value(0));
            assert_eq!(&1, MinMax::default().new_min_value(1));

            let mut min = MinMax::default();
            min.new_min_value(1);
            min.new_min_value(0);
            assert_eq!(0, min.min);

            let mut min = MinMax::default();
            min.new_min_value(1);
            min.new_min_value(2);
            assert_eq!(1, min.min);

            let mut min = MinMax::default();
            min.new_min_value(2);
            min.new_min_value(1);
            assert_eq!(1, min.min);
        }

        #[test]
        fn max() {
            assert_eq!(0, MinMax::default().max);
            assert_eq!(&0, MinMax::default().new_max_value(0));
            assert_eq!(&1, MinMax::default().new_max_value(1));

            let mut max = MinMax::default();
            max.new_max_value(1);
            max.new_max_value(0);
            assert_eq!(1, max.max);

            let mut max = MinMax::default();
            max.new_max_value(1);
            max.new_max_value(2);
            assert_eq!(2, max.max);
        }
    }
}
