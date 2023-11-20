#![doc(hidden)]
//! Base-Map for indexed read-write Maps.
//!
use std::{hash::Hash, ops::Deref};

#[cfg(feature = "hashbrown")]
use hashbrown::HashMap;

#[cfg(not(feature = "hashbrown"))]
use std::collections::HashMap;

use crate::{
    collections::{rw::Editable, Retriever},
    index::store::Store,
};

/// Is a Wrapper for an [`std::collections::HashMap`], which has trigger functions for insert and remove operations
#[repr(transparent)]
#[derive(Debug)]
pub struct TriggerMap<I, X>(HashMap<X, I>);

impl<I, X> TriggerMap<I, X>
where
    X: Hash + Eq,
{
    /// Create a `Map` with given `capacity`.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(HashMap::with_capacity(capacity))
    }

    // Return the `Item` from the given index for updating the `Item`.
    #[inline]
    pub fn get_mut(&mut self, index: &X) -> Option<&mut I> {
        self.0.get_mut(index)
    }

    /// Insert a new `Item` in the Map.
    /// If the `index` already exist, then the `insert` will be ignored!
    #[inline]
    pub fn insert<Trigger>(&mut self, index: X, item: I, mut insert: Trigger) -> bool
    where
        X: Clone,
        Trigger: FnMut(X, &I),
    {
        match self.0.get(&index) {
            Some(_) => false, // the index already exists, no insert is possible
            None => {
                insert(index.clone(), &item);
                self.0.insert(index, item);
                true
            }
        }
    }

    /// The Item in the Map will be removed.
    #[inline]
    pub fn remove<Trigger>(&mut self, index: &X, mut remove: Trigger) -> Option<I>
    where
        Trigger: FnMut(&X, &I),
    {
        let item = self.0.remove(index)?;
        remove(index, &item);
        Some(item)
    }
}

impl<I, X> Deref for TriggerMap<I, X> {
    type Target = HashMap<X, I>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

///
/// `Map` is a Map with one `Store`.
/// This means, one `Index`.
///
#[derive(Debug)]
pub struct Map<S, I, X, F> {
    field: F,
    store: S,
    items: TriggerMap<I, X>,
}

impl<S, I, X, F> Map<S, I, X, F>
where
    S: Store<Index = X>,
    F: Fn(&I) -> S::Key,
    X: Hash + Eq,
{
    pub fn new(field: F) -> Self {
        Self {
            field,
            store: S::with_capacity(0),
            items: TriggerMap::with_capacity(0),
        }
    }

    pub fn from_iter<It>(field: F, iter: It) -> Self
    where
        It: IntoIterator<Item = (X, I)> + ExactSizeIterator,
        X: Clone,
    {
        let mut s = Self {
            field,
            store: S::with_capacity(iter.len()),
            items: TriggerMap::with_capacity(iter.len()),
        };

        iter.into_iter().for_each(|(index, item)| {
            s.insert(index, item);
        });

        s
    }

    /// Insert a new `Item` to the Map.
    pub fn insert(&mut self, index: X, item: I) -> bool
    where
        X: Clone,
    {
        self.items.insert(index, item, |index, item| {
            self.store.insert((self.field)(item), index);
        })
    }

    pub fn idx(&self) -> Retriever<'_, S, HashMap<X, I>> {
        Retriever::new(&self.store, &self.items)
    }
}

impl<S, I, X, F> Editable<I> for Map<S, I, X, F>
where
    S: Store<Index = X>,
    F: Fn(&I) -> S::Key,
    X: Hash + Eq,
{
    type Key = S::Key;
    type Index = X;

    /// Update the item on the given key (index).
    fn update<U>(&mut self, index: X, mut update: U) -> Option<&I>
    where
        U: FnMut(&mut I),
    {
        self.items.get_mut(&index).map(|item| {
            let key = (self.field)(item);
            update(item);
            self.store.update(key, index, (self.field)(item));
            &*item
        })
    }

    /// The Item in the Map will be removed.
    fn remove(&mut self, index: X) -> Option<I> {
        self.items.remove(&index, |index, item| {
            self.store.delete((self.field)(item), index);
        })
    }

    fn get_indices_by_key(&self, key: &Self::Key) -> &[Self::Index] {
        self.store.get(key)
    }
}

impl<S, I, X, F> Deref for Map<S, I, X, F> {
    type Target = HashMap<X, I>;

    fn deref(&self) -> &Self::Target {
        &self.items.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::MultiIntIndex;
    use rstest::{fixture, rstest};

    #[derive(PartialEq, Debug, Clone)]
    struct Person {
        id: i32,
        name: String,
    }

    impl Person {
        fn new(id: i32, name: &str) -> Self {
            Self {
                id,
                name: name.into(),
            }
        }
    }

    #[fixture]
    fn v() -> HashMap<&'static str, Person> {
        let mut m = HashMap::new();
        m.insert("Paul", Person::new(0, "Paul"));
        m.insert("Mario", Person::new(-2, "Mario"));
        m.insert("Jasmin", Person::new(2, "Jasmin"));

        m
    }

    #[rstest]
    fn check_map(v: HashMap<&'static str, Person>) {
        let mut m = Map::<MultiIntIndex<i32, &'static str>, Person, _, _>::from_iter(
            |p| p.id,
            v.into_iter(),
        );
        assert!(m.insert("Mrs X", Person::new(-3, "Mrs X")));

        assert!(m.idx().contains(&-2));
        assert!(m.idx().contains(&-3));

        assert!(!m.idx().contains(&-1));

        // remove
        assert_eq!(4, m.len());
        assert_eq!(Person::new(-3, "Mrs X"), m.remove("Mrs X").unwrap());
        assert!(!m.idx().contains(&-3));
        assert_eq!(3, m.len());
        assert_eq!(Some(&Person::new(2, "Jasmin")), m.idx().get(&2).next());

        // update
        assert_eq!(
            Some(&Person::new(2, "Jasmin 2")),
            m.update("Jasmin", |p| {
                p.name = String::from("Jasmin 2");
            })
        );
        assert_eq!(Some(&Person::new(2, "Jasmin 2")), m.idx().get(&2).next());

        assert_eq!(["Jasmin"], m.get_indices_by_key(&2));
    }

    #[test]
    fn invalid_insert() {
        let mut m = Map::<MultiIntIndex<i32, &'static str>, Person, _, _>::new(|p| p.id);
        assert!(m.insert("Mrs X", Person::new(-3, "Mrs X")));
        // invalid insert, same index
        assert!(!m.insert("Mrs X", Person::new(-3, "Mrs X")));
        assert_eq!(1, m.len());
    }
}
