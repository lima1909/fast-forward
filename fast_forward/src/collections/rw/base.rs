//! Base-List for indexed read-write lists.
//!
use std::ops::Deref;

#[repr(transparent)]
pub struct List<I> {
    items: Vec<I>,
}

impl<I> List<I> {
    /// Create a `List` with given `capacity`.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
        }
    }

    // Return the `Item` from the given index for updating the `Item`.
    #[inline]
    pub fn get_mut(&mut self, pos: usize) -> Option<&mut I> {
        self.items.get_mut(pos)
    }

    /// Append a new `Item` to the List.
    #[inline]
    pub fn push<Trigger>(&mut self, item: I, mut insert: Trigger) -> usize
    where
        Trigger: FnMut(&I, usize),
    {
        let idx = self.items.len();
        insert(&item, idx);
        self.items.push(item);
        idx
    }

    /// The Item in the list will be removed.
    #[inline]
    pub fn remove<Trigger>(&mut self, pos: usize, mut trigger: Trigger) -> Option<I>
    where
        Trigger: FnMut(RemoveTriggerKind, &I, usize),
    {
        use RemoveTriggerKind::*;

        if self.items.is_empty() {
            return None;
        }

        let last_idx = self.items.len() - 1;
        // index out of bound
        if pos > last_idx {
            return None;
        }

        // last item in the list
        if pos == last_idx {
            let rm_item = self.items.remove(pos);
            trigger(Delete, &rm_item, pos);
            return Some(rm_item);
        }

        // remove item and entry in store and swap with last item
        let rm_item = self.items.swap_remove(pos);
        trigger(Delete, &rm_item, pos);

        // formerly last item, now item on pos
        let curr_item = &self.items[pos];
        trigger(Delete, curr_item, last_idx); // remove formerly entry in store
        trigger(Insert, curr_item, pos);

        Some(rm_item)
    }
}

pub enum RemoveTriggerKind {
    Delete,
    Insert,
}

impl<I> Deref for List<I> {
    type Target = [I];

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl<I> crate::index::Indexable<usize> for List<I> {
    type Output = I;

    fn item(&self, idx: &usize) -> &Self::Output {
        &self.items[*idx]
    }
}

impl<I> Default for List<I> {
    fn default() -> Self {
        Self { items: Vec::new() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::{fixture, rstest};

    impl<T> From<Vec<T>> for List<T> {
        fn from(v: Vec<T>) -> Self {
            Self { items: v }
        }
    }

    #[fixture]
    pub fn v() -> List<String> {
        let v: List<String> = vec![String::from("A"), String::from("B"), String::from("C")].into();
        v
    }

    #[test]
    fn check_methods() {
        let mut l = List::default();
        assert_eq!(
            0,
            l.push("A", |i, x| {
                assert_eq!(&"A", i);
                assert_eq!(0, x);
            })
        );

        let i = l.get_mut(0).unwrap();
        *i = "B"; // update
        assert_eq!(&"B", i);
        assert_eq!(&"B", l.first().unwrap());

        assert_eq!(1, l.len());

        let i = l.remove(0, |_, _, _| {});
        assert_eq!("B", i.unwrap());
        assert_eq!(0, l.len());
    }

    #[rstest]
    fn insert_trigger(mut v: List<String>) {
        let mut call_trigger_pos = 0usize;
        assert_eq!(
            3,
            v.push(String::from("D"), |_, pos| {
                call_trigger_pos += pos;
            })
        );
        assert_eq!(3, call_trigger_pos);
    }

    #[rstest]
    fn update(mut v: List<String>) {
        assert_eq!(Some(&String::from("A")), v.get(0));

        // update: "A" -> "AA" => (1, 2)
        let s = v.get_mut(0).unwrap();
        *s = String::from("AA");
        assert_eq!(Some(&String::from("AA")), v.get(0));
    }

    #[rstest]
    fn update_not_found(mut v: List<String>) {
        assert!(v.get_mut(10_000).is_none());
    }

    #[rstest]
    fn update_deleted_item(mut v: List<String>) {
        assert_eq!(&"A", &v[0]);
        v.remove(0, |_, _, _| {});
        assert_eq!(&"C", &v[0]);
    }

    // #[rstest]
    // fn drop_trigger(mut v: List<String>) {
    //     let mut call_trigger_pos = 0usize;
    //     v.remove(1, |_, _, _| {
    //         call_trigger_pos += 1;
    //     });
    //     assert_eq!(1, call_trigger_pos);
    // }

    #[rstest]
    fn remove_no_trigger(mut v: List<String>) {
        let mut call_trigger_pos = 0usize;
        v.remove(1000, |_, _, _| {
            call_trigger_pos += 1000;
        });
        assert_eq!(0, call_trigger_pos);
    }

    #[rstest]
    fn remove_first(mut v: List<String>) {
        assert_eq!(String::from("A"), v.remove(0, |_, _, _| {}).unwrap());

        assert_eq!(2, v.len());
        assert!(!v.is_empty());
        assert_eq!(&String::from("C"), v.get(0).unwrap());

        let mut it = v.iter();
        assert_eq!(Some(&"C".into()), it.next());
        assert_eq!(Some(&"B".into()), it.next());
        assert_eq!(None, it.next());
    }

    #[rstest]
    fn drop_mid(mut v: List<String>) {
        assert_eq!(String::from("B"), v.remove(1, |_, _, _| {}).unwrap());

        assert_eq!(2, v.len());
        assert!(!v.is_empty());
        assert_eq!(&String::from("C"), v.get(1).unwrap());

        let mut it = v.iter();
        assert_eq!(Some(&"A".into()), it.next());
        assert_eq!(Some(&"C".into()), it.next());
        assert_eq!(None, it.next());
    }

    #[rstest]
    fn drop_last(mut v: List<String>) {
        assert_eq!(String::from("C"), v.remove(2, |_, _, _| {}).unwrap());

        assert_eq!(2, v.len());
        assert_eq!(None, v.get(2));

        let mut it = v.iter();
        assert_eq!(Some(&"A".into()), it.next());
        assert_eq!(Some(&"B".into()), it.next());
        assert_eq!(None, it.next());
    }

    #[rstest]
    fn delete_bad_index(mut v: List<String>) {
        assert_eq!(None, v.remove(1000, |_, _, _| {}));
    }
}
