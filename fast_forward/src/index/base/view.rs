use std::ops::Deref;

use crate::index::{
    indices::{KeyIndex, MultiKeyIndex},
    store::Filterable,
};

pub trait ViewCreator<'a, F: Filterable> {
    fn create_view<It>(&'a self, keys: It) -> View<F>
    where
        It: IntoIterator<Item = F::Key>;
}

#[repr(transparent)]
pub struct View<F: Filterable>(pub(crate) F);

impl<F: Filterable> Deref for View<F> {
    type Target = F;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<X> Filterable for Vec<Option<&MultiKeyIndex<X>>>
where
    X: Ord + PartialEq,
{
    type Key = usize;
    type Index = X;

    fn contains(&self, key: &Self::Key) -> bool {
        matches!(
            <[Option<&MultiKeyIndex<X>>]>::get(self, *key),
            Some(Some(_))
        )
    }

    fn get(&self, key: &Self::Key) -> &[Self::Index] {
        match <[Option<&MultiKeyIndex<X>>]>::get(self, *key) {
            Some(Some(idxs)) => (*idxs).as_slice(),
            _ => &[],
        }
    }
}
