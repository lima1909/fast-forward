//! Base module for `Collections`.
#![allow(dead_code)]

/// `Retain` is like a normal [`std::vec::Vec`] which store the given `Items`.
/// If an `Item` was deleted, then is the `Position (Index)` of this `Item` is saved
/// and the `Position` is no longer vaild and therewith is the `Item` no longer reachable.
pub struct Retain<T> {
    items: Vec<T>,
    droped: Vec<usize>,
}

impl<T> Retain<T> {
    /// Get the Item on the given position/index in the List.
    /// If the Item was deleted, the return value is `None`
    pub fn get(&self, pos: usize) -> Option<&T> {
        if self.is_droped(pos) {
            return None;
        }
        self.items.get(pos)
    }

    /// The Item in the list will be marked as deleted.
    ///
    /// # Panics
    ///
    /// Panics if index is out of bounds.
    pub fn drop(&mut self, pos: usize) -> &T {
        let item = &self.items[pos];
        if !self.is_droped(pos) {
            self.droped.push(pos);
        }
        item
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

    // Returns all removed `Items`.
    pub fn droped_items(&self) -> impl Iterator<Item = &'_ T> {
        self.droped.iter().map(|i| &self.items[*i])
    }

    /// The number of not deleted `Items` in the List.
    pub fn count(&self) -> usize {
        self.items.len() - self.droped.len()
    }

    /// Len == 0 or Len == deleted `Items`
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

    /// Create a new `Retain` instance without all `droped Items`.
    /// Hint: The `Items` get eventual a new `Index` (position in the Vec).
    pub fn reorg(mut self) -> Self {
        self.items = self
            .items
            .into_iter()
            .enumerate()
            .filter_map(|(pos, item)| {
                if !self.droped.contains(&pos) {
                    Some(item)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        self.droped = Vec::new();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl<T> From<Vec<T>> for Retain<T> {
        fn from(v: Vec<T>) -> Self {
            Self {
                items: v,
                droped: Vec::new(),
            }
        }
    }

    #[test]
    fn len_count_empty() {
        let mut v = Retain::with_capacity(2);
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
        let mut v: Retain<_> = vec![String::from("A"), String::from("B"), String::from("C")].into();

        assert_eq!(&String::from("A"), v.drop(0));

        assert_eq!(3, v.len());
        assert_eq!(2, v.count());
        assert!(!v.is_empty());
        assert!(v.is_droped(0));
        assert_eq!(None, v.get(0));
        assert_eq!(&[0], v.droped_indices());
        assert_eq!(
            vec![&String::from("A")],
            v.droped_items().collect::<Vec<_>>()
        );

        let mut it = v.iter();
        assert_eq!(Some(&"B".into()), it.next());
        assert_eq!(Some(&"C".into()), it.next());
        assert_eq!(None, it.next());
    }

    #[test]
    fn drop_mid() {
        let mut v: Retain<_> = vec![String::from("A"), String::from("B"), String::from("C")].into();

        assert_eq!(&String::from("B"), v.drop(1));

        assert_eq!(3, v.len());
        assert_eq!(2, v.count());
        assert!(!v.is_empty());
        assert!(v.is_droped(1));
        assert_eq!(None, v.get(1));
        assert_eq!(&[1], v.droped_indices());
        assert_eq!(
            vec![&String::from("B")],
            v.droped_items().collect::<Vec<_>>()
        );

        let mut it = v.iter();
        assert_eq!(Some(&"A".into()), it.next());
        assert_eq!(Some(&"C".into()), it.next());
        assert_eq!(None, it.next());
    }

    #[test]
    fn drop_last() {
        let mut v: Retain<_> = vec![String::from("A"), String::from("B"), String::from("C")].into();

        assert_eq!(&String::from("C"), v.drop(2));

        assert_eq!(3, v.len());
        assert_eq!(2, v.count());
        assert!(!v.is_empty());
        assert!(v.is_droped(2));
        assert_eq!(None, v.get(2));
        assert_eq!(&[2], v.droped_indices());
        assert_eq!(
            vec![&String::from("C")],
            v.droped_items().collect::<Vec<_>>()
        );

        let mut it = v.iter();
        assert_eq!(Some(&"A".into()), it.next());
        assert_eq!(Some(&"B".into()), it.next());
        assert_eq!(None, it.next());
    }

    #[test]
    #[should_panic]
    fn delete_index_panic() {
        let mut v: Retain<_> = vec![String::from("A"), String::from("B"), String::from("C")].into();
        v.drop(1000);
    }

    #[test]
    fn is_empty() {
        let mut v: Retain<_> = vec![String::from("A"), String::from("B"), String::from("C")].into();
        assert!(!v.is_empty());
        assert_eq!(Some(&"A".into()), v.get(0));

        assert_eq!(&String::from("A"), v.drop(0));

        assert_eq!(2, v.count());
        assert_eq!(3, v.len());
        assert!(!v.is_empty());

        // drop again 0
        assert_eq!(&String::from("A"), v.drop(0));
        assert_eq!(2, v.count());
        assert_eq!(3, v.len());
        assert!(!v.is_empty());

        assert_eq!(&String::from("B"), v.drop(1));
        assert_eq!(1, v.count());
        assert_eq!(3, v.len());
        assert!(!v.is_empty());

        assert_eq!(&String::from("C"), v.drop(2));
        assert_eq!(0, v.count());
        assert_eq!(3, v.len());
        assert!(v.is_empty());
    }

    #[test]
    fn reorg() {
        let mut v: Retain<_> = vec![String::from("A"), String::from("B"), String::from("C")].into();
        assert_eq!(&String::from("B"), v.drop(1));
        v = v.reorg();
        assert_eq!(vec![String::from("A"), String::from("C")], v.items);
        assert_eq!(2, v.len());
        assert_eq!(2, v.count());

        let mut v: Retain<_> = vec![String::from("A"), String::from("B"), String::from("C")].into();
        assert_eq!(&String::from("A"), v.drop(0));
        assert_eq!(&String::from("C"), v.drop(2));
        v = v.reorg();
        assert_eq!(vec![String::from("B")], v.items);
        assert_eq!(1, v.len());
        assert_eq!(1, v.count());

        let mut v: Retain<_> = vec![String::from("A"), String::from("B"), String::from("C")].into();
        assert_eq!(&String::from("A"), v.drop(0));
        assert_eq!(&String::from("C"), v.drop(2));
        assert_eq!(&String::from("B"), v.drop(1));
        v = v.reorg();
        assert!(v.is_empty());
        assert_eq!(0, v.len());
        assert_eq!(0, v.count());
    }
}
