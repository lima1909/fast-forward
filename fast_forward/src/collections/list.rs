use crate::index::Indexable;

#[derive(Debug)]
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
    pub fn delete<F>(&mut self, pos: usize, mut trigger: F) -> Option<&T>
    where
        F: FnMut(&T, &usize), // param are: &Item, current position in the list
    {
        let del_item = self.items.get(pos)?;
        trigger(del_item, &pos);

        self.deleted_pos.push(pos);
        Some(del_item)
    }

    /// Get the Item on the given position/index in the List.
    /// If the Item was deleted, the return value is `None`
    pub fn get(&self, pos: usize) -> Option<&T> {
        if self.is_deleted(pos) {
            return None;
        }
        self.items.get(pos)
    }

    /// Check, is the Item on `pos` (`Index`) deleted.
    #[inline]
    pub fn is_deleted(&self, pos: usize) -> bool {
        self.deleted_pos.contains(&pos)
    }

    // Returns all removed `Indices`.
    pub fn deleted_indices(&self) -> &[usize] {
        &self.deleted_pos
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

    /// Returns an `Iterator` over all not deleted `Items`.
    pub const fn iter(&self) -> Iter<'_, T> {
        Iter::new(self)
    }

    /// Create a `List` with given `capacity`.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
            deleted_pos: Vec::new(),
        }
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

impl<T> Indexable<usize> for List<T> {
    type Output = T;

    fn item(&self, idx: &usize) -> &Self::Output {
        if self.is_deleted(*idx) {
            panic!("Item on index: '{idx}' was deleted");
        }
        &self.items[*idx]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::indices::Indices;

    impl<T> From<Vec<T>> for List<T> {
        fn from(v: Vec<T>) -> Self {
            let mut l = List::with_capacity(v.len());
            for i in v {
                l.insert(i, |_, _| {});
            }
            l
        }
    }

    #[test]
    fn insert() {
        let mut l = List::default();
        assert_eq!(0, l.len());
        assert_eq!(0, l.count());
        assert!(l.is_empty());

        assert_eq!(0, l.insert("A", |_, _| {}));
        assert_eq!(1, l.insert("B", |_, _| {}));

        assert_eq!(2, l.len());
        assert_eq!(2, l.count());
        assert!(!l.is_empty());
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

        assert_eq!(Some(&"C"), l.get(0));
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

        let mut it = l.iter();
        assert_eq!(Some(&1), it.next());
        assert_eq!(Some(&2), it.next());
        assert_eq!(Some(&3), it.next());
        assert_eq!(None, it.next());

        assert_eq!(Some(&2), l.get(1));
        assert_eq!(&3, l.item(&2)); // get with Index
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

        assert_eq!(Some(&1), l.delete(0, |_, _| {}));
        assert_eq!(3, l.len());
        assert_eq!(2, l.count());
        assert_eq!(&[0usize], l.deleted_indices());

        assert!(l.is_deleted(0));
        assert!(!l.is_deleted(1));
        assert!(!l.is_deleted(2));
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
        assert_eq!(&[1usize], l.deleted_indices());

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
        assert_eq!(&[2usize], l.deleted_indices());

        assert!(!l.is_deleted(0));
        assert!(!l.is_deleted(1));
        assert!(l.is_deleted(2));

        let mut it = l.iter();
        assert_eq!(Some(&1), it.next());
        assert_eq!(Some(&2), it.next());
        assert_eq!(None, it.next());
    }

    #[test]
    fn insert_after_delete_last() {
        let mut l: List<_> = vec![1, 2, 3].into();

        l.delete(2, |_, _| {});
        assert_eq!(3, l.len());
        assert_eq!(2, l.count());
        assert_eq!(&[2usize], l.deleted_indices());

        l.insert(5, |_, _| {});
        assert_eq!(4, l.len());
        assert_eq!(3, l.count());

        let mut it = l.iter();
        assert_eq!(Some(&1), it.next());
        assert_eq!(Some(&2), it.next());
        assert_eq!(Some(&5), it.next());
        assert_eq!(None, it.next());
    }

    #[test]
    #[should_panic]
    fn delete_index_panic() {
        let mut l: List<_> = vec![1, 2, 3].into();
        l.delete(0, |_, _| {});
        assert_eq!(&1, l.item(&0));
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
    #[should_panic]
    fn iter_filter_delete() {
        let mut l: List<_> = vec![1, 2, 3].into();
        l.delete(1, |_, _| {});

        let idx: Indices = [1].into();
        let mut it = idx.items(&l);
        assert_eq!(Some(&1), it.next());
        assert_eq!(None, it.next());
    }

    #[test]
    fn filter_first() {
        let l: List<_> = vec![1, 2, 3].into();
        let idx: Indices = [1, 0].into();
        let mut it = idx.items(&l);

        assert_eq!(Some(&1), it.next());
        assert_eq!(Some(&2), it.next());
        assert_eq!(None, it.next());
    }

    #[test]
    fn filter_last() {
        let l: List<_> = vec![1, 2, 3].into();
        let idx: Indices = [2, 1].into();
        let mut it = idx.items(&l);

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
