//! Base module for `Collections`.
use std::ops::Index;

use crate::index::Indexable;

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

    /// Get a mutable Item on the given position/index in the List.
    /// If the Item was deleted, the return value is `None`
    pub fn get_mut(&mut self, pos: usize) -> Option<&mut T> {
        if self.is_droped(pos) {
            return None;
        }
        self.items.get_mut(pos)
    }

    /// Insert a new `Item` to the List.
    pub fn insert<F>(&mut self, item: T, mut trigger: F) -> usize
    where
        F: FnMut(&T, usize),
    {
        let pos = self.items.len();
        trigger(&item, pos);
        self.items.push(item);
        pos
    }

    /// The Item in the list will be marked as deleted.
    pub fn drop<F>(&mut self, pos: usize, mut trigger: F) -> Option<&T>
    where
        F: FnMut(&T),
    {
        let item = self.items.get(pos)?;
        if !self.is_droped(pos) {
            trigger(item);
            self.droped.push(pos);
        }
        Some(item)
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

impl<T> Indexable<usize> for Retain<T> {
    type Output = T;

    fn item(&self, idx: &usize) -> &Self::Output {
        if self.is_droped(*idx) {
            panic!("Item on index: '{idx}' was deleted");
        }
        &self.items[*idx]
    }
}

impl<T> Index<usize> for Retain<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.items[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::{fixture, rstest};

    impl<T> From<Vec<T>> for Retain<T> {
        fn from(v: Vec<T>) -> Self {
            Self {
                items: v,
                droped: Vec::new(),
            }
        }
    }

    #[fixture]
    pub fn v() -> Retain<String> {
        let v: Retain<String> =
            vec![String::from("A"), String::from("B"), String::from("C")].into();
        v
    }

    #[test]
    fn len_count_empty() {
        let mut v = Retain::with_capacity(2);
        assert_eq!(0, v.len());
        assert_eq!(0, v.count());
        assert!(v.is_empty());

        assert_eq!(0, v.insert("A", |_, _| {}));
        assert_eq!(1, v.insert("B", |_, _| {}));

        assert_eq!(2, v.len());
        assert_eq!(2, v.count());
        assert!(!v.is_empty());

        let mut it = v.iter();
        assert_eq!(Some(&"A"), it.next());
        assert_eq!(Some(&"B"), it.next());
        assert_eq!(None, it.next());
    }

    #[rstest]
    #[should_panic]
    fn get_item_not_found(v: Retain<String>) {
        v.item(&10000);
    }

    #[rstest]
    #[should_panic]
    fn get_droped_item(mut v: Retain<String>) {
        v.drop(1, |_| {});
        v.item(&1);
    }

    #[rstest]
    fn insert_trigger(mut v: Retain<String>) {
        let mut call_trigger_pos = 0usize;
        assert_eq!(
            3,
            v.insert(String::from("D"), |_, pos| {
                call_trigger_pos += pos;
            })
        );
        assert_eq!(3, call_trigger_pos);
    }

    #[rstest]
    fn update(mut v: Retain<String>) {
        assert_eq!(Some(&String::from("A")), v.get(0));

        // update: "A" -> "AA" => (1, 2)
        let s = v.get_mut(0).unwrap();
        *s = String::from("AA");
        assert_eq!(Some(&String::from("AA")), v.get(0));
    }

    #[rstest]
    fn update_not_found(mut v: Retain<String>) {
        assert!(v.get_mut(10_000).is_none());
    }

    #[rstest]
    fn update_deleted_item(mut v: Retain<String>) {
        assert!(v.get(0).is_some());
        v.drop(0, |_| {});
        assert!(v.get_mut(0).is_none());
    }

    #[rstest]
    fn drop_trigger(mut v: Retain<String>) {
        let mut call_trigger_pos = 0usize;
        v.drop(1, |_| {
            call_trigger_pos += 1;
        });
        assert_eq!(1, call_trigger_pos);
    }

    #[rstest]
    fn drop_no_trigger(mut v: Retain<String>) {
        let mut call_trigger_pos = 0usize;
        v.drop(1000, |_| {
            call_trigger_pos += 1000;
        });
        assert_eq!(0, call_trigger_pos);
    }

    #[rstest]
    fn drop_first(mut v: Retain<String>) {
        assert_eq!(Some(&String::from("A")), v.drop(0, |_| {}));

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

    #[rstest]
    fn drop_mid(mut v: Retain<String>) {
        assert_eq!(Some(&String::from("B")), v.drop(1, |_| {}));

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

    #[rstest]
    fn drop_last(mut v: Retain<String>) {
        assert_eq!(Some(&String::from("C")), v.drop(2, |_| {}));

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

    #[rstest]
    fn delete_bad_index(mut v: Retain<String>) {
        assert_eq!(None, v.drop(1000, |_| {}));
    }

    #[rstest]
    fn is_empty(mut v: Retain<String>) {
        assert!(!v.is_empty());
        assert_eq!(Some(&"A".into()), v.get(0));

        assert_eq!(Some(&String::from("A")), v.drop(0, |_| {}));

        assert_eq!(2, v.count());
        assert_eq!(3, v.len());
        assert!(!v.is_empty());

        // drop again 0
        assert_eq!(Some(&String::from("A")), v.drop(0, |_| {}));
        assert_eq!(2, v.count());
        assert_eq!(3, v.len());
        assert!(!v.is_empty());

        assert_eq!(Some(&String::from("B")), v.drop(1, |_| {}));
        assert_eq!(1, v.count());
        assert_eq!(3, v.len());
        assert!(!v.is_empty());

        assert_eq!(Some(&String::from("C")), v.drop(2, |_| {}));
        assert_eq!(0, v.count());
        assert_eq!(3, v.len());
        assert!(v.is_empty());
    }

    #[test]
    fn reorg_drop_1() {
        let mut l = v();
        assert_eq!(Some(&String::from("B")), l.drop(1, |_| {}));
        l = l.reorg();
        assert_eq!(vec![String::from("A"), String::from("C")], l.items);
        assert_eq!(2, l.len());
        assert_eq!(2, l.count());
    }

    #[test]
    fn reorg_drop_0_2() {
        let mut l = v();
        assert_eq!(Some(&String::from("A")), l.drop(0, |_| {}));
        assert_eq!(Some(&String::from("C")), l.drop(2, |_| {}));
        l = l.reorg();
        assert_eq!(vec![String::from("B")], l.items);
        assert_eq!(1, l.len());
        assert_eq!(1, l.count());
        assert!(!l.is_empty());
    }

    #[test]
    fn reorg_drop_0_1_2() {
        let mut l = v();
        assert_eq!(Some(&String::from("A")), l.drop(0, |_| {}));
        assert_eq!(Some(&String::from("C")), l.drop(2, |_| {}));
        assert_eq!(Some(&String::from("B")), l.drop(1, |_| {}));
        l = l.reorg();
        assert!(l.is_empty());
        assert_eq!(0, l.len());
        assert_eq!(0, l.count());
        assert!(l.is_empty());
    }
}
