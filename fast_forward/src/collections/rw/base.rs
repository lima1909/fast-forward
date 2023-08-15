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

    /// Append a new `Item` to the List.
    #[inline]
    pub fn push<Triger>(&mut self, item: I, mut insert: Triger)
    where
        Triger: FnMut(&I, usize),
    {
        let idx = self.items.len();
        insert(&item, idx);
        self.items.push(item);
    }

    /// Update the item on the given position.
    #[inline]
    pub fn update<U, Triger>(&mut self, pos: usize, mut update: U, mut before: Triger) -> Option<&I>
    where
        U: FnMut(&mut I),
        Triger: FnMut(&I),
    {
        self.items.get_mut(pos).map(|item| {
            before(item);
            update(item);
            &*item
        })
    }

    /// The Item in the list will be removed.
    #[inline]
    pub fn remove<TrigerDel, TrigerIns>(
        &mut self,
        pos: usize,
        mut delete: TrigerDel,
        mut insert: TrigerIns,
    ) -> Option<I>
    where
        TrigerDel: FnMut(&I, &usize),
        TrigerIns: FnMut(&I, usize),
    {
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
            // self.store.delete((self.field)(&rm_item), &pos);
            delete(&rm_item, &pos);
            return Some(rm_item);
        }

        // remove item and entry in store and swap with last item
        let rm_item = self.items.swap_remove(pos);
        delete(&rm_item, &pos);

        // formerly last item, now item on pos
        let curr_item = &self.items[pos];
        delete(curr_item, &last_idx); // remove formerly entry in store
        insert(curr_item, pos);

        Some(rm_item)
    }
}

impl<I> Deref for List<I> {
    type Target = [I];

    fn deref(&self) -> &Self::Target {
        &self.items
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

        let i = l.update(0, |i| *i = "B", |_i| {});
        assert_eq!(&"B", i.unwrap());
        assert_eq!(1, l.len());

        let i = l.remove(0, |_, _| {}, |_, _| {});
        assert_eq!("B", i.unwrap());
        assert_eq!(0, l.len());
    }
}
