//! Query combines different filter. Filters can be linked using `and` and `or`.
use crate::{error, index::Filterable, Idx, Predicate, Result};
use std::{
    borrow::Cow,
    cmp::{min, Ordering::*},
};

pub const EMPTY_IDXS: &[Idx] = &[];

pub trait Queryable<'k>: Sized {
    /// Filter is the fastest way to ask with one [`Predicate`].
    fn filter<P>(&self, p: P) -> Result<Cow<[usize]>>
    where
        P: Into<Predicate<'k>>;

    /// Query combined different `filter` with an logical `or` | `and`.
    fn query<P>(&self, p: P) -> Query<Self>
    where
        P: Into<Predicate<'k>>,
    {
        match self.filter(p.into()) {
            Ok(idxs) => Query::new(self, idxs),
            Err(err) => Query::new_err(self, err),
        }
    }
}

impl<'k, F: Filterable<'k>> Queryable<'k> for F {
    fn filter<P>(&self, p: P) -> Result<Cow<[usize]>>
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
    err: Option<error::Error>,
}

impl<'q, Q> Query<'q, Q> {
    fn new(q: &'q Q, first: Cow<'q, [usize]>) -> Self {
        Self {
            q,
            first,
            ors: vec![],
            err: None,
        }
    }

    fn new_err(q: &'q Q, err: error::Error) -> Self {
        Self {
            q,
            first: Cow::Borrowed(EMPTY_IDXS),
            ors: vec![],
            err: Some(err),
        }
    }
}

impl<'k, 'q, Q> Query<'q, Q>
where
    Q: Queryable<'k>,
{
    pub fn or<P>(mut self, p: P) -> Self
    where
        P: Into<Predicate<'k>>,
    {
        if let Some(_err) = &self.err {
            return self;
        }

        match self.q.filter(p.into()) {
            Ok(idxs) => self.ors.push(idxs),
            Err(err) => self.err = Some(err),
        }
        self
    }

    pub fn and<P>(mut self, p: P) -> Self
    where
        P: Into<Predicate<'k>>,
    {
        if let Some(_err) = &self.err {
            return self;
        }

        match self.q.filter(p.into()) {
            Ok(idxs) => {
                if self.ors.is_empty() {
                    self.first = and(&self.first, &idxs);
                } else {
                    let i = self.ors.len() - 1;
                    self.ors[i] = and(&self.ors[i], &idxs);
                }
            }
            Err(err) => self.err = Some(err),
        }
        self
    }

    #[must_use = "query do nothing, before execute"]
    pub fn exec(mut self) -> Result<Cow<'q, [usize]>> {
        if let Some(err) = self.err {
            return Err(err);
        }

        for next in self.ors {
            self.first = or(self.first, next);
        }
        Ok(self.first)
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
        use crate::eq;

        use super::*;

        impl<'k> Queryable<'k> for Vec<i32> {
            fn filter<P>(&self, p: P) -> Result<Cow<[usize]>>
            where
                P: Into<Predicate<'k>>,
            {
                let p = p.into();
                let pos: usize = p.2.try_into()?;

                let idxs = match self.get(pos) {
                    Some(_) => Cow::Owned(vec![pos]),
                    None => Cow::Borrowed(EMPTY_IDXS),
                };

                Ok(idxs)
            }
        }

        fn values() -> Vec<i32> {
            vec![0, 1, 2, 3]
        }

        #[test]
        fn filter() -> Result {
            assert_eq!(1, values().filter(eq("", 1))?[0]);
            assert_eq!(EMPTY_IDXS, &*values().filter(eq("", 99))?);

            Ok(())
        }

        #[test]
        fn and() -> Result {
            assert_eq!(1, values().query(eq("", 1)).and(eq("", 1)).exec()?[0]);
            assert_eq!(
                EMPTY_IDXS,
                &*values().query(eq("", 1)).and(eq("", 2)).exec()?
            );

            Ok(())
        }

        #[test]
        fn or() -> Result {
            assert_eq!([1, 2], *values().query(eq("", 1)).or(eq("", 2)).exec()?);
            assert_eq!([1], &*values().query(eq("", 1)).or(eq("", 99)).exec()?);
            assert_eq!([1], &*values().query(eq("", 99)).or(eq("", 1)).exec()?);

            Ok(())
        }

        #[test]
        fn and_or() -> Result {
            // (1 and 1) or 2 => [1, 2]
            assert_eq!(
                [1, 2],
                *values()
                    .query(eq("", 1))
                    .and(eq("", 1))
                    .or(eq("", 2))
                    .exec()?
            );
            // (1 and 2) or 3 => [3]
            assert_eq!(
                [3],
                *values()
                    .query(eq("", 1))
                    .and(eq("", 2))
                    .or(eq("", 3))
                    .exec()?
            );

            Ok(())
        }

        #[test]
        fn or_and_12() -> Result {
            // 1 or (2 and 2) => [1, 2]
            assert_eq!(
                [1, 2],
                *values()
                    .query(eq("", 1))
                    .or(eq("", 2))
                    .and(eq("", 2))
                    .exec()?
            );
            // 1 or (3 and 2) => [1]
            assert_eq!(
                [1],
                *values()
                    .query(eq("", 1))
                    .or(eq("", 3))
                    .and(eq("", 2))
                    .exec()?
            );

            Ok(())
        }

        #[test]
        fn or_and_3() -> Result {
            // 3 or (2 and 1) => [3]
            assert_eq!(
                [3],
                *values()
                    .query(eq("", 3))
                    .or(eq("", 2))
                    .and(eq("", 1))
                    .exec()?
            );

            Ok(())
        }

        #[test]
        fn and_or_and_2() -> Result {
            // (2 and 2) or (2 and 1) => [2]
            assert_eq!(
                [2],
                *values()
                    .query(eq("", 2))
                    .and(eq("", 2))
                    .or(eq("", 2))
                    .and(eq("", 1))
                    .exec()?
            );

            Ok(())
        }

        #[test]
        fn and_or_and_03() -> Result {
            // 0 or (1 and 2) or 3) => [0, 3]
            assert_eq!(
                [0, 3],
                *values()
                    .query(eq("", 0))
                    .or(eq("", 1))
                    .and(eq("", 2))
                    .or(eq("", 3))
                    .exec()?
            );

            Ok(())
        }

        #[test]
        fn error() {
            let v = values();
            let err = v.query(eq("", "AA")).or(eq("", 1)).and(eq("", 2)).exec();
            assert!(err.is_err());

            let err = v.query(eq("", 0)).or(eq("", "AA")).and(eq("", 2)).exec();
            assert!(err.is_err());

            let err = v.query(eq("", 0)).or(eq("", 1)).and(eq("", "AA")).exec();
            assert!(err.is_err());
        }
    }
}
