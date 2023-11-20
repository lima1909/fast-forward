//! read-write collections.
//!
pub mod list;
pub mod list_base;
pub mod map_base;

pub use list::IList;

use std::marker::PhantomData;

/// `Editable` describe the operations for changing (update and remove) `Items` in a collection.
pub trait Editable<I> {
    type Key;
    type Index;

    /// Update the item on the given position.
    fn update<U>(&mut self, index: Self::Index, update: U) -> Option<&I>
    where
        U: FnMut(&mut I);

    /// The Item on the given position will be removed from the list.
    fn remove(&mut self, index: Self::Index) -> Option<I>;

    // Get all `Indices` by a given `Key`
    fn get_indices_by_key(&self, key: &Self::Key) -> &[Self::Index];
}

/// `Editor` used a given`Editable` to execute change operation by `Key` instead of an `Index`.
pub struct Editor<'a, I, E> {
    editor: &'a mut E,
    _items: PhantomData<I>,
}

impl<'a, I, E> Editor<'a, I, E>
where
    E: Editable<I, Index = usize>,
{
    pub fn new(editor: &'a mut E) -> Self {
        Self {
            editor,
            _items: PhantomData,
        }
    }

    /// Call `update`-function of all items by a given `Key`.
    pub fn update_by_key<U>(&mut self, key: &E::Key, mut update: U)
    where
        U: FnMut(&mut I),
    {
        #[allow(clippy::unnecessary_to_owned)]
        for idx in self.editor.get_indices_by_key(key).to_vec() {
            self.editor.update(idx, &mut update);
        }
    }

    /// Call `update`-function of all items by a given `Key`,
    /// with a given callback for getting a reference to the updated `Item(s)`.
    pub fn update_by_key_with_cb<U, C>(&mut self, key: &E::Key, mut update: U, mut callback: C)
    where
        U: FnMut(&mut I),
        C: FnMut(&I),
    {
        #[allow(clippy::unnecessary_to_owned)]
        for idx in self.editor.get_indices_by_key(key).to_vec() {
            if let Some(item) = self.editor.update(idx, &mut update) {
                callback(item);
            }
        }
    }

    /// Remove all items by a given `Key`.
    pub fn remove_by_key(&mut self, key: &E::Key) {
        while let Some(idx) = self.editor.get_indices_by_key(key).iter().next() {
            self.editor.remove(*idx);
        }
    }

    /// Remove all items by a given `Key`, with a given callback for getting the removed `Item(s)`.
    pub fn remove_by_key_with_cb<C>(&mut self, key: &E::Key, mut callback: C)
    where
        C: FnMut(I),
    {
        while let Some(idx) = self.editor.get_indices_by_key(key).iter().next() {
            if let Some(item) = self.editor.remove(*idx) {
                callback(item);
            }
        }
    }
}
