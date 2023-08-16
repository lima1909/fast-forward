//! Base-List for indexed read-write lists.
//!
use std::ops::Deref;

#[derive(Default)]
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

    #[inline]
    pub fn get_mut(&mut self, pos: usize) -> Option<&mut I> {
        self.items.get_mut(pos)
    }

    /// Append a new `Item` to the List.
    #[inline]
    pub fn push<Trigger>(&mut self, item: I, mut insert: Trigger)
    where
        Trigger: FnMut(&I, usize),
    {
        let idx = self.items.len();
        insert(&item, idx);
        self.items.push(item);
    }

    /// Update the item on the given position.
    #[inline]
    pub fn update<U, Trigger, After, Keys>(
        &mut self,
        pos: usize,
        mut update: U,
        before: Trigger,
    ) -> Option<&I>
    where
        U: FnMut(&mut I),
        Trigger: for<'a> Fn(&'a I) -> (Keys, After),
        After: for<'a> FnOnce(Keys, &'a I),
    {
        self.items.get_mut(pos).map(|item| {
            let (keys, after) = before(item);
            update(item);
            after(keys, item);
            &*item
        })
    }

    /// The Item in the list will be removed.
    #[inline]
    pub fn remove<Trigger>(&mut self, pos: usize, mut trigger: Trigger) -> Option<I>
    where
        Trigger: FnMut(RemoveTrigger, &I, usize),
    {
        use RemoveTrigger::*;

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

pub enum RemoveTrigger {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_methods() {
        let mut l = List::default();
        l.push("A", |i, x| {
            assert_eq!(&"A", i);
            assert_eq!(0, x);
        });

        let i = l.update(
            0,
            |i| *i = "B", // update
            |i| {
                assert_eq!(&"A", i); // before trigger
                ((1, String::from("XYZ")), |keys, i| {
                    // after trigger, with Keys: 1 and "XYZ"
                    assert_eq!((1, String::from("XYZ")), keys);
                    assert_eq!(&"B", i);
                })
            },
        );
        assert_eq!(&"B", i.unwrap());
        assert_eq!(1, l.len());

        let i = l.remove(0, |_, _, _| {});
        assert_eq!("B", i.unwrap());
        assert_eq!(0, l.len());
    }
}
