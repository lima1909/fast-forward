use std::{
    collections::{btree_map::Entry, BTreeMap},
    fmt::Debug,
};

use crate::{ops, Filter, Idx, IdxFilter};

use super::{Index, KeyIdxStore};

#[derive(Debug, Default)]
pub struct StrMapIndex<'a, I: Index>(BTreeMap<&'a str, I>);

impl<'a, I: Index> IdxFilter<&str> for StrMapIndex<'a, I> {
    fn idx(&self, f: Filter<&str>) -> &[Idx] {
        if f.op != ops::EQ {
            return &[];
        }

        match self.0.get(f.key) {
            Some(i) => i.get(),
            None => &[],
        }
    }
}

impl<'a, I: Index> KeyIdxStore<&'a str> for StrMapIndex<'a, I> {
    fn insert(&mut self, k: &'a str, i: Idx) -> super::Result {
        match self.0.entry(k) {
            Entry::Vacant(e) => {
                e.insert(I::new(i));
                Ok(())
            }
            Entry::Occupied(mut e) => e.get_mut().add(i),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ops::eq, Query};

    mod unique {
        use crate::index::{IndexError, Unique};

        use super::*;

        #[test]
        fn empty() {
            let i = StrMapIndex::<Unique>::default();
            assert_eq!(0, i.idx(eq("Jasmin")).len());
            assert!(i.0.is_empty());
        }

        #[test]
        fn find_idx_2() {
            let mut i = StrMapIndex::<Unique>::default();
            i.insert("Jasmin", 4).unwrap();

            assert_eq!(i.idx(eq("Jasmin")), &[4]);
            assert_eq!(1, i.0.len());
        }

        #[test]
        fn or_find_idx_3_4() {
            let mut i = StrMapIndex::<Unique>::default();
            i.insert("Jasmin", 4).unwrap();
            i.insert("Mario", 8).unwrap();
            i.insert("Paul", 6).unwrap();

            let r = i.or(eq("Mario"), eq("Paul"));
            assert!(r.contains(&&8));
            assert!(r.contains(&&6));

            let r = i.or(eq("Paul"), eq("Blub"));
            assert!(r.contains(&&6));

            let r = i.or(eq("Blub"), eq("Mario"));
            assert!(r.contains(&&8));
        }

        #[test]
        fn double_index() {
            let mut i = StrMapIndex::<Unique>::default();
            i.insert("Jasmin", 2).unwrap();

            assert_eq!(Err(IndexError::NotUniqueKey), i.insert("Jasmin", 2));
        }

        #[test]
        fn out_of_bound() {
            let i = StrMapIndex::<Unique>::default();
            assert_eq!(0, i.filter(eq("Jasmin")).len());
        }
    }

    mod multi {
        use crate::index::Multi;

        use super::*;

        #[test]
        fn empty() {
            let i = StrMapIndex::<Multi>::default();
            assert_eq!(0, i.idx(eq("Jasmin")).len());
            assert!(i.0.is_empty());
        }

        #[test]
        fn find_idx_2() {
            let mut i = StrMapIndex::<Multi>::default();
            i.insert("Jasmin", 2).unwrap();

            assert!(i.idx(eq("Jasmin")).eq(&[2]));
            assert_eq!(1, i.0.len());
        }

        #[test]
        fn double_index() {
            let mut i = StrMapIndex::<Multi>::default();
            i.insert("Jasmin", 2).unwrap();
            i.insert("Jasmin", 1).unwrap();

            assert!(i.filter(eq("Jasmin")).eq(&[2, 1]));
        }
    }
}
