pub mod list;
pub mod one;

pub use one::OneIndexList;

use crate::index::SelectedIndices;

/// `ListIndexFilter` means, that you get an `Iterator` over all `Items` which exists for a given list of indices.
pub trait ListIndexFilter {
    type Item;

    /// Returns `Some(Item)` from the given index (position) if it exist, otherwise `None`
    fn item(&self, index: usize) -> Option<&Self::Item>;

    /// Returns a `Iterator` over all `Items` with the given index list.
    fn filter<'i>(&'i self, indices: SelectedIndices<'i>) -> Iter<'i, Self>
    where
        Self: Sized,
    {
        Iter::new(self, indices)
    }
}

pub struct Iter<'i, F: ListIndexFilter> {
    pos: usize,
    list: &'i F,
    indices: SelectedIndices<'i>,
}

impl<'i, F> Iter<'i, F>
where
    F: ListIndexFilter,
{
    pub const fn new(list: &'i F, indices: SelectedIndices<'i>) -> Self {
        Self {
            pos: 0,
            list,
            indices,
        }
    }
}

impl<'i, F> Iterator for Iter<'i, F>
where
    F: ListIndexFilter,
{
    type Item = &'i F::Item;

    fn next(&mut self) -> Option<Self::Item> {
        while self.pos < self.indices.len() {
            let idx = self.indices[self.pos];
            self.pos += 1;
            match self.list.item(idx) {
                Some(item) => return Some(item),
                // ignore deleted items
                None => continue,
            }
        }
        None
    }
}
