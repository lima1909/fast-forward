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
    fn insert(&mut self, item: T) {
        // self.interceptor.insert(&item, self.len());
        self.items.push(item);
    }

    fn delete(&mut self, pos: usize) {
        // let del_item = &self.items[pos];
        // self.interceptor.delete(&del_item, self.len());
        self.items.remove(pos);
        self.deleted_pos.push(pos);
    }

    fn is_deleted(&self, pos: usize) -> bool {
        self.deleted_pos.contains(&pos)
    }

    fn get(&self, pos: usize) -> Option<&T> {
        let item = self.items.get(pos)?;
        if self.is_deleted(pos) {
            return None;
        }
        Some(item)
    }

    fn len(&self) -> usize {
        self.items.len()
    }

    fn filter<'i>(&'i self, filter: Cow<'i, [usize]>) -> FilterIter<'i, T> {
        FilterIter::new(filter, &self)
    }

    fn iter(&self) -> Iter<'_, T> {
        Iter::new(&self)
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

                return self.list.get(self.pos);
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
        let idx = self.filter[self.pos];
        self.pos += 1;
        return Some(&self.list[idx]);
    }
}
