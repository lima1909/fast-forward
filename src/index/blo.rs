use std::borrow::Cow;

use crate::Idx;

pub const EMPTY: &[Idx] = &[];

pub fn or<'a>(lhs: &'a [Idx], rhs: &'a [Idx]) -> Cow<'a, [Idx]> {
    match (lhs.is_empty(), rhs.is_empty()) {
        (false, false) => {
            use std::cmp::Ordering::*;

            let ll = lhs.len();
            let lr = rhs.len();

            let mut li = 0;
            let mut ri = 0;

            let mut v = Vec::with_capacity(ll + lr);

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
                    break;
                } else if lr == ri {
                    v.extend(lhs[li..].iter());
                    break;
                }
            }

            Cow::Owned(v)
        }
        (true, false) => Cow::Borrowed(rhs),
        (false, true) => Cow::Borrowed(lhs),
        (true, true) => Cow::Borrowed(EMPTY),
    }
}

pub fn and<'a>(lhs: &'a [Idx], rhs: &'a [Idx]) -> Cow<'a, [Idx]> {
    let ll = lhs.len();
    let lr = rhs.len();

    // if ll == 0 || lr == 0 {
    //     Cow::Borrowed(EMPTY)
    // } else {
    let len = if ll > lr { ll } else { lr };

    let mut v = Vec::with_capacity(len);
    let mut found = 0;

    for l in lhs {
        #[allow(clippy::needless_range_loop)]
        for i in found..lr {
            let r = rhs[i];

            #[allow(clippy::comparison_chain)]
            if l < &r {
                break;
            } else if l == &r {
                v.push(r);
                found += 1;
                break;
            }
        }
    }

    Cow::Owned(v)
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod or {
        use super::*;

        #[test]
        fn both_empty() {
            assert_eq!(EMPTY, &*or(EMPTY, EMPTY));
        }

        #[test]
        fn only_left() {
            assert_eq!([1, 2], *or(&[1, 2], EMPTY));
        }

        #[test]
        fn only_right() {
            assert_eq!([1, 2], *or(EMPTY, &[1, 2]));
        }

        #[test]
        fn diff_len() {
            assert_eq!([1, 2, 3], *or(&[1], &[2, 3]),);
            assert_eq!([1, 2, 3], *or(&[2, 3], &[1]),);
        }

        #[test]
        fn overlapping_simple() {
            assert_eq!([1, 2, 3], *or(&[1, 2], &[2, 3]),);
            assert_eq!([1, 2, 3], *or(&[2, 3], &[1, 2]),);
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
