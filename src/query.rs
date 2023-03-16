//! Query combines different filter. Filters can be linked using `and` and `or`.
use crate::{index::Filterable, Idx, Predicate};
use std::{
    borrow::Cow,
    cmp::{min, Ordering::*},
};

pub const EMPTY: &[Idx] = &[];

pub trait Queryable<'k>: Sized {
    /// Filter is the fastes way to ask with one [`Predicate`].
    /// For example: `pk` (name) `=` (ops::EQ) `6` (Key::Usize(6))
    fn filter<P>(&self, p: P) -> Cow<[usize]>
    where
        P: Into<Predicate<'k>>;

    /// Query combined different `filter` with an logical `or` or `and`.
    fn query<P>(&self, p: P) -> Query<Self>
    where
        P: Into<Predicate<'k>>,
    {
        let idxs = self.filter(p.into());
        Query {
            q: self,
            first: idxs,
            ors: vec![],
        }
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

/// Query combines different filter. Filters can be linked using `and` and `or`.
pub struct Query<'q, Q> {
    q: &'q Q,
    first: Cow<'q, [usize]>,
    ors: Vec<Cow<'q, [usize]>>,
}

impl<'k, 'q, Q> Query<'q, Q>
where
    Q: Queryable<'k>,
{
    pub fn or<P>(mut self, p: P) -> Self
    where
        P: Into<Predicate<'k>>,
    {
        let idxs = self.q.filter(p.into());
        self.ors.push(idxs);
        self
    }

    pub fn and<P>(mut self, p: P) -> Self
    where
        P: Into<Predicate<'k>>,
    {
        let idxs = self.q.filter(p.into());

        if self.ors.is_empty() {
            self.first = and(&self.first, &idxs);
        } else {
            let i = self.ors.len() - 1;
            self.ors[i] = and(&self.ors[i], &idxs);
        }

        self
    }

    pub fn exec(mut self) -> Cow<'q, [usize]> {
        for next in self.ors {
            self.first = or(self.first, next);
        }
        self.first
    }
}

// pub fn or<'a>(lhs: &'a [Idx], rhs: &'a [Idx]) -> Cow<'a, [Idx]> {
pub fn or<'a>(lhs: Cow<'a, [Idx]>, rhs: Cow<'a, [Idx]>) -> Cow<'a, [Idx]> {
    match (lhs.is_empty(), rhs.is_empty()) {
        (false, false) => {
            let ll = lhs.len();
            let lr = rhs.len();
            let mut v = Vec::with_capacity(ll + lr);

            let mut li = 0;
            let mut ri = 0;

            loop {
                let l = lhs[li];
                let r = rhs[ri];

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
                    v.extend(rhs[ri..].iter());
                    return Cow::Owned(v);
                } else if lr == ri {
                    v.extend(lhs[li..].iter());
                    return Cow::Owned(v);
                }
            }
        }
        (true, false) => rhs,
        (false, true) => lhs,
        (true, true) => Cow::Borrowed(EMPTY),
    }
}

pub fn and<'a>(lhs: &[Idx], rhs: &[Idx]) -> Cow<'a, [Idx]> {
    if lhs.is_empty() || rhs.is_empty() {
        return Cow::Borrowed(EMPTY);
    }

    let ll = lhs.len();
    let lr = rhs.len();
    let mut v = Vec::with_capacity(min(ll, lr));

    let mut li = 0;
    let mut ri = 0;

    loop {
        let l = lhs[li];

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

#[cfg(test)]
mod tests {
    use super::*;

    //     mod or {
    //         use super::*;

    //         #[test]
    //         fn both_empty() {
    //             assert_eq!(EMPTY, &*or(EMPTY, EMPTY));
    //         }

    //         #[test]
    //         fn only_left() {
    //             assert_eq!([1, 2], *or(&[1, 2], EMPTY));
    //         }

    //         #[test]
    //         fn only_right() {
    //             assert_eq!([1, 2], *or(EMPTY, &[1, 2]));
    //         }

    //         #[test]
    //         fn diff_len() {
    //             assert_eq!([1, 2, 3], *or(&[1], &[2, 3]),);
    //             assert_eq!([1, 2, 3], *or(&[2, 3], &[1]),);
    //         }

    //         #[test]
    //         fn overlapping_simple() {
    //             assert_eq!([1, 2, 3], *or(&[1, 2], &[2, 3]),);
    //             assert_eq!([1, 2, 3], *or(&[2, 3], &[1, 2]),);
    //         }

    //         #[test]
    //         fn overlapping_diff_len() {
    //             // 1, 2, 8, 9, 12
    //             // 2, 5, 6, 10
    //             assert_eq!(
    //                 *or(&[1, 2, 8, 9, 12], &[2, 5, 6, 10]),
    //                 [1, 2, 5, 6, 8, 9, 10, 12]
    //             );

    //             // 2, 5, 6, 10
    //             // 1, 2, 8, 9, 12
    //             assert_eq!(
    //                 *or(&[2, 5, 6, 10], &[1, 2, 8, 9, 12]),
    //                 [1, 2, 5, 6, 8, 9, 10, 12]
    //             );
    //         }
    //     }

    mod and {
        use super::*;

        #[test]
        fn both_empty() {
            assert_eq!(EMPTY, &*and(EMPTY, EMPTY));
        }

        #[test]
        fn only_left() {
            assert_eq!(EMPTY, &*and(&[1, 2], EMPTY));
        }

        #[test]
        fn only_right() {
            assert_eq!(EMPTY, &*and(EMPTY, &[1, 2]));
        }

        #[test]
        fn diff_len() {
            assert_eq!(EMPTY, &*and(&[1], &[2, 3]));
            assert_eq!(EMPTY, &*and(&[2, 3], &[1]));

            assert_eq!([2], *and(&[2], &[2, 5]));
            assert_eq!([2], *and(&[2], &[1, 2, 3]));
            assert_eq!([2], *and(&[2], &[0, 1, 2]));

            assert_eq!([2], *and(&[2, 5], &[2]));
            assert_eq!([2], *and(&[1, 2, 3], &[2]));
            assert_eq!([2], *and(&[0, 1, 2], &[2]));
        }

        #[test]
        fn overlapping_simple() {
            assert_eq!([2], *and(&[1, 2], &[2, 3]),);
            assert_eq!([2], *and(&[2, 3], &[1, 2]),);

            assert_eq!([1], *and(&[1, 2], &[1, 3]),);
            assert_eq!([1], *and(&[1, 3], &[1, 2]),);
        }

        #[test]
        fn overlapping_diff_len() {
            // 1, 2, 8, 9, 12
            // 2, 5, 6, 10
            assert_eq!([2, 12], *and(&[1, 2, 8, 9, 12], &[2, 5, 6, 10, 12, 13, 15]));

            // 2, 5, 6, 10
            // 1, 2, 8, 9, 12
            assert_eq!([2, 12], *and(&[2, 5, 6, 10, 12, 13, 15], &[1, 2, 8, 9, 12]));
        }
    }
}
