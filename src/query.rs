//! Query combines different filter. Filters can be linked using `and` and `or`.
use crate::Idx;
use std::{
    borrow::Cow,
    cmp::{min, Ordering::*},
};

pub const EMPTY_IDXS: &[Idx] = &[];

/// `query` factory for creating a `Query` with the first started filter result.
pub const fn query(idxs: Cow<[usize]>) -> Query<'_> {
    Query::new(idxs)
}

/// Query combines different filters by using `and` and `or`.
pub struct Query<'q> {
    first: Cow<'q, [usize]>,
    ors: Vec<Cow<'q, [usize]>>,
}

impl<'q> Query<'q> {
    /// Create a new `Query` with initial `Indices`.
    const fn new(first: Cow<'q, [usize]>) -> Self {
        Self { first, ors: vec![] }
    }
}

impl<'q> Query<'q> {
    /// Combine two `Indices` with an logical `OR`.
    pub fn or(mut self, idxs: Cow<'q, [usize]>) -> Self {
        self.ors.push(idxs);
        self
    }

    /// Combine two `Indices` with an logical `AND`.
    pub fn and(mut self, idxs: Cow<[usize]>) -> Self {
        if self.ors.is_empty() {
            self.first = and(&self.first, &idxs);
        } else {
            let i = self.ors.len() - 1;
            self.ors[i] = and(&self.ors[i], &idxs);
        }
        self
    }

    /// Execute all logical `OR`s.
    #[inline]
    pub fn exec(mut self) -> Cow<'q, [usize]> {
        for next in self.ors {
            self.first = or(self.first, next);
        }
        self.first
    }

    /// Execute all given filters and applay the filter to an given `Slice`.
    #[inline]
    pub fn filter<T>(self, list: &[T]) -> Vec<&T> {
        self.exec().iter().map(|i| &list[*i]).collect()
    }
}

// Logical `Or`, the union of two Inices.
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
        (true, true) => Cow::Borrowed(EMPTY_IDXS),
    }
}

// Logical `And`, the intersection of two Inices.
pub fn and<'a>(lhs: &[Idx], rhs: &[Idx]) -> Cow<'a, [Idx]> {
    if lhs.is_empty() || rhs.is_empty() {
        return Cow::Borrowed(EMPTY_IDXS);
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

    mod or {
        use crate::{query::EMPTY_IDXS, Idx};
        use std::borrow::Cow;

        pub fn or<'a>(lhs: &'a [Idx], rhs: &'a [Idx]) -> Cow<'a, [Idx]> {
            super::or(Cow::Borrowed(lhs), Cow::Borrowed(rhs))
        }

        #[test]
        fn both_empty() {
            assert_eq!(EMPTY_IDXS, &*or(EMPTY_IDXS, EMPTY_IDXS));
        }

        #[test]
        fn only_left() {
            assert_eq!([1, 2], *or(&[1, 2], EMPTY_IDXS));
        }

        #[test]
        fn only_right() {
            assert_eq!([1, 2], *or(EMPTY_IDXS, &[1, 2]));
        }

        #[test]
        fn diff_len() {
            assert_eq!([1, 2, 3], *or(&[1], &[2, 3]));
            assert_eq!([1, 2, 3], *or(&[2, 3], &[1]));
        }

        #[test]
        fn overlapping_simple() {
            assert_eq!([1, 2, 3], *or(&[1, 2], &[2, 3]));
            assert_eq!([1, 2, 3], *or(&[2, 3], &[1, 2]));
        }

        #[test]
        fn overlapping_diff_len() {
            // 1, 2, 8, 9, 12
            // 2, 5, 6, 10
            assert_eq!(
                *or(&[1, 2, 8, 9, 12], &[2, 5, 6, 10]),
                [1, 2, 5, 6, 8, 9, 10, 12]
            );

            // 2, 5, 6, 10
            // 1, 2, 8, 9, 12
            assert_eq!(
                *or(&[2, 5, 6, 10], &[1, 2, 8, 9, 12]),
                [1, 2, 5, 6, 8, 9, 10, 12]
            );
        }
    }

    mod and {
        use super::*;

        #[test]
        fn both_empty() {
            assert_eq!(EMPTY_IDXS, &*and(EMPTY_IDXS, EMPTY_IDXS));
        }

        #[test]
        fn only_left() {
            assert_eq!(EMPTY_IDXS, &*and(&[1, 2], EMPTY_IDXS));
        }

        #[test]
        fn only_right() {
            assert_eq!(EMPTY_IDXS, &*and(EMPTY_IDXS, &[1, 2]));
        }

        #[test]
        fn diff_len() {
            assert_eq!(EMPTY_IDXS, &*and(&[1], &[2, 3]));
            assert_eq!(EMPTY_IDXS, &*and(&[2, 3], &[1]));

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

    mod query {
        use super::*;

        struct List(Vec<i32>);

        impl List {
            fn eq(&self, i: i32) -> Cow<[Idx]> {
                match self.0.binary_search(&i) {
                    Ok(pos) => Cow::Owned(vec![pos]),
                    Err(_) => Cow::Borrowed(EMPTY_IDXS),
                }
            }
        }

        fn values() -> List {
            List(vec![0, 1, 2, 3])
        }

        #[test]
        fn filter() {
            let l = values();
            assert_eq!(1, l.eq(1)[0]);
            assert_eq!(EMPTY_IDXS, &*values().eq(99));
        }

        #[test]
        fn and() {
            let l = values();
            assert_eq!(1, query(l.eq(1)).and(l.eq(1)).exec()[0]);
            assert_eq!(EMPTY_IDXS, &*query(l.eq(1)).and(l.eq(2)).exec());
        }

        #[test]
        fn or() {
            let l = values();
            assert_eq!([1, 2], *query(l.eq(1)).or(l.eq(2)).exec());
            assert_eq!([1], &*query(l.eq(1)).or(l.eq(99)).exec());
            assert_eq!([1], &*query(l.eq(99)).or(l.eq(1)).exec());
        }

        #[test]
        fn and_or() {
            let l = values();
            // (1 and 1) or 2 => [1, 2]
            assert_eq!([1, 2], *query(l.eq(1)).and(l.eq(1)).or(l.eq(2)).exec());
            // (1 and 2) or 3 => [3]
            assert_eq!([3], *query(l.eq(1)).and(l.eq(2)).or(l.eq(3)).exec());
        }

        #[test]
        fn or_and_12() {
            let l = values();
            // 1 or (2 and 2) => [1, 2]
            assert_eq!([1, 2], *query(l.eq(1)).or(l.eq(2)).and(l.eq(2)).exec());
            // 1 or (3 and 2) => [1]
            assert_eq!([1], *query(l.eq(1)).or(l.eq(3)).and(l.eq(2)).exec());
        }

        #[test]
        fn or_and_3() {
            let l = values();
            // 3 or (2 and 1) => [3]
            assert_eq!([3], *query(l.eq(3)).or(l.eq(2)).and(l.eq(1)).exec());
        }

        #[test]
        fn and_or_and_2() {
            let l = values();
            // (2 and 2) or (2 and 1) => [2]
            assert_eq!(
                [2],
                *query(l.eq(2)).and(l.eq(2)).or(l.eq(2)).and(l.eq(1)).exec()
            );
        }

        #[test]
        fn and_or_and_03() {
            let l = values();
            // 0 or (1 and 2) or 3) => [0, 3]
            assert_eq!(
                [0, 3],
                *query(l.eq(0)).or(l.eq(1)).and(l.eq(2)).or(l.eq(3)).exec()
            );
        }
    }
}
