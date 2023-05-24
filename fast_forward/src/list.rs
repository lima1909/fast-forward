use crate::ListIndexFilter;
use std::ops::Index;

#[derive(Debug, Clone)]
pub struct List<T> {
    items: Vec<T>,
    deleted_pos: Vec<usize>,
}

/// List for saving Items with trigger by insert, update and delete, to inform e.g. `Store` to update the `Index`.
impl<T> List<T> {
    /// Insert the given item  and return the inserted position in the list.
    pub fn insert<F>(&mut self, item: T, mut trigger: F) -> usize
    where
        F: FnMut(&T, usize), // param are: &Item, position in the list after inserting
    {
        let pos = self.items.len();
        trigger(&item, pos);

        self.items.push(item);
        pos
    }

    /// Update the item on the given position.
    ///
    /// # Panics
    ///
    /// Panics if the pos is out of bound.
    ///
    pub fn update<U, F>(&mut self, pos: usize, update_fn: U, mut trigger: F) -> bool
    where
        U: Fn(&T) -> T,
        F: FnMut(&T, usize, &T), // param are: (old) &Item, current position in the list, (new) &Item
    {
        match self.items.get(pos) {
            Some(old) => {
                let new = (update_fn)(old);
                trigger(old, pos, &new);

                self.items[pos] = new;
                true
            }
            None => false,
        }
    }

    /// The Item in the list will not be delteted. It will be marked as deleted.
    ///
    /// # Panics
    ///
    /// Panics if the pos is out of bound.
    ///
    pub fn delete<F>(&mut self, pos: usize, mut trigger: F) -> &T
    where
        F: FnMut(&T, usize), // param are: &Item, current position in the list
    {
        let del_item = &self.items[pos];
        trigger(del_item, pos);

        self.deleted_pos.push(pos);
        del_item
    }

    pub fn is_deleted(&self, pos: usize) -> bool {
        self.deleted_pos.contains(&pos)
    }

    /// The number of not deleted Items in the List.
    pub fn count(&self) -> usize {
        self.items.len() - self.deleted_pos.len()
    }

    /// Len == 0 or Len == deleted Items
    pub fn is_empty(&self) -> bool {
        self.items.len() == self.deleted_pos.len()
    }

    /// The length of the List (including the deleted Items).
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub const fn iter(&self) -> Iter<'_, T> {
        Iter::new(self)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
            deleted_pos: Vec::new(),
        }
    }
}

impl<T> ListIndexFilter for List<T> {
    type Item = T;

    /// Get the Item on the given position in the List. If the Item was deleted, the return is `None`
    fn item(&self, index: usize) -> Option<&Self::Item> {
        if self.is_deleted(index) {
            return None;
        }
        self.items.get(index)
    }
}

impl<T> Default for List<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            deleted_pos: Vec::new(),
        }
    }
}

impl<T> From<Vec<T>> for List<T> {
    fn from(v: Vec<T>) -> Self {
        let mut l = List::with_capacity(v.len());
        for i in v {
            l.insert(i, |_, _| {});
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

pub struct Iter<'i, T> {
    pos: usize,
    list: &'i List<T>,
}

impl<'i, T> Iter<'i, T> {
    pub const fn new(list: &'i List<T>) -> Self {
        Self { pos: 0, list }
    }
}

impl<'i, T> Iterator for Iter<'i, T> {
    type Item = &'i T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.list.item(self.pos) {
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

                let ret = self.list.item(self.pos);
                self.pos += 1;
                return ret;
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;

    #[test]
    fn insert() {
        let mut l = List::default();
        assert_eq!(0, l.len());
        assert_eq!(0, l.count());
        assert!(l.is_empty());

        assert_eq!(0, l.insert("A", |_, _| {}));
        assert_eq!(1, l.insert("B", |_, _| {}));
    }

    #[test]
    fn insert_trigger() {
        let mut l = List::default();
        l.insert("A", |_, _| {});

        let mut call_trigger_pos = 0usize;
        assert_eq!(
            1,
            l.insert("B", |_, pos| {
                call_trigger_pos += pos;
            })
        );
        assert_eq!(1, call_trigger_pos);
    }

    #[test]
    fn update() {
        let mut l = List::default();

        assert_eq!(0, l.insert("A", |_, _| {}));
        assert_eq!(1, l.insert("B", |_, _| {}));

        assert!(l.update(0, |_| "C", |_, _, _| {}));
        assert!(!l.update(100, |_| "C", |_, _, _| {}));
    }

    #[test]
    fn update_trigger() {
        let mut l = List::default();
        assert_eq!(0, l.insert("A", |_, _| {}));
        assert_eq!(1, l.insert("B", |_, _| {}));

        let mut call_trigger_pos = 0usize;
        assert!(l.update(
            1,
            |_| "C",
            |_, pos, _| {
                call_trigger_pos += pos;
            }
        ));
        assert_eq!(1, call_trigger_pos);
    }

    #[test]
    fn get() {
        let l: List<_> = vec![1, 2, 3].into();
        assert_eq!(3, l.len());
        assert_eq!(3, l.count());

        assert_eq!(Some(&1), l.iter().next());
        assert_eq!(Some(&2), l.item(1));
        assert_eq!(3, l[2]); // get with Index
    }

    #[test]
    fn delete_trigger() {
        let mut l: List<_> = vec![1, 2, 3].into();

        let mut call_trigger_pos = 0usize;
        l.delete(1, |_, pos| {
            call_trigger_pos += pos;
        });
        assert_eq!(1, call_trigger_pos);
    }

    #[test]
    fn delete_first() {
        let mut l: List<_> = vec![1, 2, 3].into();

        assert_eq!(&1, l.delete(0, |_, _| {}));
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

        l.delete(1, |_, _| {});
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

        l.delete(2, |_, _| {});
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
        l.delete(0, |_, _| {});
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
    fn iter_filter_delete() {
        let mut l: List<_> = vec![1, 2, 3].into();
        l.delete(1, |_, _| {});

        let mut it = l.filter(Cow::Owned(vec![0, 1, 2]));

        assert_eq!(Some(&1), it.next());
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

    #[test]
    fn is_empty() {
        let mut l: List<_> = vec![1].into();
        assert!(!l.is_empty());

        l.delete(0, |_, _| {});
        assert_eq!(0, l.count());
        assert!(l.is_empty());
        assert_eq!(1, l.len());
    }
}
