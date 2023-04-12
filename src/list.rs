#![allow(dead_code)]

use std::{borrow::Cow, ops::Index};

trait Interceptor<T> {
    fn insert(&mut self, item: &T, pos: usize);
    fn delete(&mut self, item: &T, pos: usize);
}

#[derive(Debug, Default, Clone)]
struct List<T> {
    items: Vec<T>,
    deleted_pos: Vec<usize>,
    // interceptor: I,
}

impl<T> List<T> {
    /// Insert the given item  and return the inserted position in the list.
    fn insert(&mut self, item: T) -> usize {
        // self.interceptor.insert(&item, self.len());
        let pos = self.items.len();
        self.items.push(item);
        pos
    }

    /// Update the item on the given position.
    ///
    /// # Panics
    ///
    /// Panics if the pos is out of bound.
    ///
    fn update<F>(&mut self, pos: usize, update_fn: F) -> bool
    where
        F: Fn(&T) -> T,
    {
        match self.items.get(pos) {
            Some(old) => {
                self.items[pos] = (update_fn)(old);
                true
            }
            None => false,
        }
    }

    /// The Item in the list will not be delteted. It will be marked as deleted.
    fn delete(&mut self, pos: usize) {
        // let del_item = &self.items[pos];
        // self.interceptor.delete(&del_item, self.len());
        self.deleted_pos.push(pos);
    }

    fn is_deleted(&self, pos: usize) -> bool {
        self.deleted_pos.contains(&pos)
    }

    /// Get the Item on the given position in the List. If the Item was deleted, the return `get` -> `None`
    fn get(&self, pos: usize) -> Option<&T> {
        let item = self.items.get(pos)?;
        if self.is_deleted(pos) {
            return None;
        }
        Some(item)
    }

    /// The number of not deleted Items in the List.
    fn count(&self) -> usize {
        self.items.len() - self.deleted_pos.len()
    }

    /// The length of the List (including the deleted Items).
    fn len(&self) -> usize {
        self.items.len()
    }

    fn filter<'i>(&'i self, filter: Cow<'i, [usize]>) -> FilterIter<'i, T> {
        FilterIter::new(filter, self)
    }

    fn iter(&self) -> Iter<'_, T> {
        Iter::new(self)
    }
}

impl<T: Default> From<Vec<T>> for List<T> {
    fn from(v: Vec<T>) -> Self {
        let mut l = List::default();
        for i in v {
            l.insert(i);
        }
        l
    }
}

impl<T> Index<usize> for List<T> {
    type Output = T;

    fn index(&self, pos: usize) -> &Self::Output {
        if self.is_deleted(pos) {
            panic!("Item is deleted");
        }
        &self.items[pos]
    }
}

struct Iter<'i, T> {
    pos: usize,
    list: &'i List<T>,
}

impl<'i, T> Iter<'i, T> {
    pub fn new(list: &'i List<T>) -> Self {
        Self { pos: 0, list }
    }
}

impl<'i, T> Iterator for Iter<'i, T> {
    type Item = &'i T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.list.get(self.pos) {
            Some(item) => {
                self.pos += 1;
                Some(item)
            }
            None => loop {
                if self.pos == self.list.len() {
                    return None;
                }

                if self.list.is_deleted(self.pos) {
                    self.pos += 1;
                    continue;
                }

                let ret = self.list.get(self.pos);
                self.pos += 1;
                return ret;
            },
        }
    }
}

struct FilterIter<'i, T> {
    pos: usize,
    filter: Cow<'i, [usize]>,
    list: &'i List<T>,
}

impl<'i, T> FilterIter<'i, T> {
    pub fn new(filter: Cow<'i, [usize]>, list: &'i List<T>) -> Self {
        Self {
            pos: 0,
            filter,
            list,
        }
    }
}

impl<'i, T> Iterator for FilterIter<'i, T> {
    type Item = &'i T;

    fn next(&mut self) -> Option<Self::Item> {
        while self.pos < self.filter.len() {
            let idx = self.filter[self.pos];
            self.pos += 1;
            if self.list.is_deleted(idx) {
                continue;
            }
            return Some(&self.list[idx]);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert() {
        let mut l = List::default();
        assert_eq!(0, l.len());
        assert_eq!(0, l.count());

        assert_eq!(0, l.insert("A"));
        assert_eq!(1, l.insert("B"));
    }

    #[test]
    fn update() {
        let mut l = List::default();

        assert_eq!(0, l.insert("A"));
        assert_eq!(1, l.insert("B"));

        assert!(l.update(0, |_| "C"));
        assert!(!l.update(100, |_| "C"));
    }

    #[test]
    fn get() {
        let l: List<_> = vec![1, 2, 3].into();
        assert_eq!(3, l.len());
        assert_eq!(3, l.count());

        assert_eq!(Some(&1), l.iter().next());
        assert_eq!(Some(&2), l.get(1));
        assert_eq!(3, l[2]); // get with Index
    }

    #[test]
    fn delete_first() {
        let mut l: List<_> = vec![1, 2, 3].into();

        l.delete(0);
        assert_eq!(3, l.len());
        assert_eq!(2, l.count());

        assert!(l.is_deleted(0));
        assert!(!l.is_deleted(1));
        assert!(!l.is_deleted(99));

        let mut it = l.iter();
        assert_eq!(Some(&2), it.next());
        assert_eq!(Some(&3), it.next());
        assert_eq!(None, it.next());
    }

    #[test]
    fn delete_mid() {
        let mut l: List<_> = vec![1, 2, 3].into();

        l.delete(1);
        assert_eq!(3, l.len());
        assert_eq!(2, l.count());

        assert!(!l.is_deleted(0));
        assert!(l.is_deleted(1));
        assert!(!l.is_deleted(2));

        let mut it = l.iter();
        assert_eq!(Some(&1), it.next());
        assert_eq!(Some(&3), it.next());
        assert_eq!(None, it.next());
    }

    #[test]
    fn delete_last() {
        let mut l: List<_> = vec![1, 2, 3].into();

        l.delete(2);
        assert_eq!(3, l.len());
        assert_eq!(2, l.count());

        assert!(!l.is_deleted(0));
        assert!(!l.is_deleted(1));
        assert!(l.is_deleted(2));

        let mut it = l.iter();
        assert_eq!(Some(&1), it.next());
        assert_eq!(Some(&2), it.next());
        assert_eq!(None, it.next());
    }

    #[test]
    #[should_panic]
    fn delete_index_panic() {
        let mut l: List<_> = vec![1, 2, 3].into();
        l.delete(0);
        assert_eq!(1, l[0]);
    }

    #[test]
    fn iter() {
        let l: List<_> = vec![1, 2, 3].into();
        let mut it = l.iter();

        assert_eq!(Some(&1), it.next());
        assert_eq!(Some(&2), it.next());
        assert_eq!(Some(&3), it.next());
        assert_eq!(None, it.next());
    }

    #[test]
    fn filter_first() {
        let l: List<_> = vec![1, 2, 3].into();
        let mut it = l.filter(Cow::Owned(vec![0, 1]));

        assert_eq!(Some(&1), it.next());
        assert_eq!(Some(&2), it.next());
        assert_eq!(None, it.next());
    }

    #[test]
    fn filter_last() {
        let l: List<_> = vec![1, 2, 3].into();
        let mut it = l.filter(Cow::Owned(vec![1, 2]));

        assert_eq!(Some(&2), it.next());
        assert_eq!(Some(&3), it.next());
        assert_eq!(None, it.next());
    }
}
