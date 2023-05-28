pub mod list;
pub mod one;

use std::ops::Index;

pub use one::OneIndexList;

use crate::index::SelectedIndices;

/// `IndexFilter` means, that you get an `Iterator` over all `Items` which exists for a given list of indices.
pub trait IndexFilter: Index<usize, Output = Self::Item> {
    type Item;

    /// Returns a `Iterator` over all `Items` with the given index list.
    fn filter<'i>(&'i self, indices: SelectedIndices<'i>) -> Iter<'i, Self>
    where
        Self: Sized,
    {
        Iter::new(self, indices)
    }
}

pub struct Iter<'i, F> {
    pos: usize,
    list: &'i F,
    indices: SelectedIndices<'i>,
}

impl<'i, F> Iter<'i, F> {
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
    F: IndexFilter,
{
    type Item = &'i F::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.indices.get(self.pos)?;
        self.pos += 1;
        Some(&self.list[*idx])
    }
}
