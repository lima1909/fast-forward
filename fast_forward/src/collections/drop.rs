#![allow(dead_code)]

/// `DropVec` is like a normal Vec which store the given `Items`.
/// If an `Item` was deleted, then is the `Position (Index)` of this `Item` saved
/// and the `Position` is no longer vaild and the `Item`is no longer reachable.

#[derive(Default)]
pub struct DropVec<T> {
    items: Vec<T>,
    droped: Vec<usize>,
}

impl<T> DropVec<T> {
    /// The Item in the list will be marked as deleted.
    ///
    /// # Panics
    ///
    /// Panics if index is out of bounds.
    pub fn drop<F>(&mut self, pos: usize, mut trigger: F) -> &T
    where
        F: FnMut(&T, &usize), // param are: &Item, current position in the list
    {
        let item = &self.items[pos];
        if !self.droped.contains(&pos) {
            trigger(item, &pos);
            self.droped.push(pos);
        }
        item
    }

    /// Get the Item on the given position/index in the List.
    /// If the Item was deleted, the return value is `None`
    pub fn get(&self, pos: usize) -> Option<&T> {
        if self.is_droped(pos) {
            return None;
        }
        self.items.get(pos)
    }

    /// Check, is the Item on `pos` (`Index`) deleted.
    #[inline]
    pub fn is_droped(&self, pos: usize) -> bool {
        self.droped.contains(&pos)
    }

    // Returns all removed `Indices`.
    pub fn droped_indices(&self) -> &[usize] {
        &self.droped
    }

    /// The number of not deleted Items in the List.
    pub fn count(&self) -> usize {
        self.items.len() - self.droped.len()
    }

    /// Len == 0 or Len == deleted Items
    pub fn is_empty(&self) -> bool {
        self.items.len() == self.droped.len()
    }

    /// The length of the List (including the deleted Items).
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Create a `List` with given `capacity`.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
            droped: Vec::new(),
        }
    }

    /// Returns an `Iterator` over all not deleted `Items`.
    pub fn iter(&self) -> impl Iterator<Item = &'_ T> {
        self.items.iter().enumerate().filter_map(|(pos, item)| {
            if !self.droped.contains(&pos) {
                Some(item)
            } else {
                None
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl<T> From<Vec<T>> for DropVec<T> {
        fn from(v: Vec<T>) -> Self {
            Self {
                items: v,
                droped: Vec::new(),
            }
        }
    }

    #[test]
    fn len_count_empty() {
        let mut v = DropVec::with_capacity(2);
        assert_eq!(0, v.len());
        assert_eq!(0, v.count());
        assert!(v.is_empty());

        v.items.push("A");
        v.items.push("B");

        assert_eq!(2, v.len());
        assert_eq!(2, v.count());
        assert!(!v.is_empty());

        let mut it = v.iter();
        assert_eq!(Some(&"A"), it.next());
        assert_eq!(Some(&"B"), it.next());
        assert_eq!(None, it.next());
    }

    #[test]
    fn drop_first() {
        let mut v: DropVec<String> =
            vec![String::from("A"), String::from("B"), String::from("C")].into();

        v.drop(0, |_, _| {});

        assert_eq!(3, v.len());
        assert_eq!(2, v.count());
        assert!(!v.is_empty());
        assert!(v.is_droped(0));
        assert_eq!(&[0], v.droped_indices());
        assert_eq!(None, v.get(0));

        let mut it = v.iter();
        assert_eq!(Some(&"B".into()), it.next());
        assert_eq!(Some(&"C".into()), it.next());
        assert_eq!(None, it.next());
    }

    #[test]
    fn drop_mid() {
        let mut v: DropVec<String> =
            vec![String::from("A"), String::from("B"), String::from("C")].into();

        v.drop(1, |_, _| {});

        assert_eq!(3, v.len());
        assert_eq!(2, v.count());
        assert!(!v.is_empty());
        assert!(v.is_droped(1));
        assert_eq!(&[1], v.droped_indices());
        assert_eq!(None, v.get(1));

        let mut it = v.iter();
        assert_eq!(Some(&"A".into()), it.next());
        assert_eq!(Some(&"C".into()), it.next());
        assert_eq!(None, it.next());
    }

    #[test]
    fn drop_last() {
        let mut v: DropVec<String> =
            vec![String::from("A"), String::from("B"), String::from("C")].into();

        v.drop(2, |_, _| {});

        assert_eq!(3, v.len());
        assert_eq!(2, v.count());
        assert!(!v.is_empty());
        assert!(v.is_droped(2));
        assert_eq!(&[2], v.droped_indices());
        assert_eq!(None, v.get(2));

        let mut it = v.iter();
        assert_eq!(Some(&"A".into()), it.next());
        assert_eq!(Some(&"B".into()), it.next());
        assert_eq!(None, it.next());
    }

    #[test]
    fn delete_trigger() {
        let mut v: DropVec<String> =
            vec![String::from("A"), String::from("B"), String::from("C")].into();

        let mut call_trigger_pos = 0usize;
        v.drop(1, |_, pos| {
            call_trigger_pos += pos;
        });
        assert_eq!(1, call_trigger_pos);

        // no trigger for drop the same pos
        v.drop(1, |_, pos| {
            call_trigger_pos += pos;
        });
        assert_eq!(1, call_trigger_pos);
    }

    #[test]
    #[should_panic]
    fn delete_index_panic() {
        let mut v: DropVec<String> =
            vec![String::from("A"), String::from("B"), String::from("C")].into();
        v.drop(1000, |_, _| {});
    }

    #[test]
    fn is_empty() {
        let mut v: DropVec<String> =
            vec![String::from("A"), String::from("B"), String::from("C")].into();
        assert!(!v.is_empty());
        assert_eq!(Some(&"A".into()), v.get(0));

        v.drop(0, |_, _| {});
        assert_eq!(2, v.count());
        assert_eq!(3, v.len());
        assert!(!v.is_empty());

        // drop again 0
        v.drop(0, |_, _| {});
        assert_eq!(2, v.count());
        assert_eq!(3, v.len());
        assert!(!v.is_empty());

        v.drop(1, |_, _| {});
        assert_eq!(1, v.count());
        assert_eq!(3, v.len());
        assert!(!v.is_empty());

        v.drop(2, |_, _| {});
        assert_eq!(0, v.count());
        assert_eq!(3, v.len());
        assert!(v.is_empty());
    }
}
