//! Different kinds of collections which are using `Indices`.
//!
#[doc(hidden)]
pub(crate) mod list;
pub mod ro;
pub mod rw;

use std::ops::Index;

pub use crate::collections::{ro::ROIndexList, rw::RWIndexList};
use crate::index::{store::Filter as StoreFilter, Filterable, Indices, MetaData, Store};

/// [`Filter`] combines a given [`Filterable`] with the given list of items.
pub struct Filter<'a, F, I> {
    filter: StoreFilter<'a, F>,
    _items: &'a I,
}

impl<'a, F, I> Filter<'a, F, I>
where
    F: Filterable,
{
    const fn new(filter: &'a F, items: &'a I) -> Self {
        Self {
            filter: StoreFilter(filter),
            _items: items,
        }
    }

    #[inline]
    pub fn eq(&self, key: &F::Key) -> Indices<'a> {
        self.filter.eq(key)
    }

    #[inline]
    pub fn contains(&self, key: &F::Key) -> bool {
        self.filter.contains(key)
    }

    #[inline]
    pub fn items(&'a self, key: &F::Key) -> impl Iterator<Item = &'a <I as Index<usize>>::Output>
    where
        I: Index<usize>,
    {
        self.filter.items(key, self._items)
    }
}

/// A `Retriever` is the interface for get Items by an given filter|query.
#[repr(transparent)]
pub struct Retriever<'a, S, I>(Filter<'a, S, I>);

impl<'a, S, I> Retriever<'a, S, I>
where
    S: Store,
{
    /// Create a new instance of an [`Retriever`].
    pub const fn new(filter: &'a S, items: &'a I) -> Self {
        Self(Filter::new(filter, items))
    }

    #[inline]
    pub fn eq(&self, key: &S::Key) -> Indices<'a> {
        self.0.eq(key)
    }

    /// Checks whether the `Key` exists.
    ///
    /// ## Example
    ///
    /// ```
    /// use fast_forward::index::{Store, uint::UIntIndex};
    /// use fast_forward::collections::ro::ROIndexList;
    ///
    /// #[derive(Debug, Eq, PartialEq, Clone)]
    /// pub struct Car(usize, String);
    ///
    /// let cars = vec![Car(2, "BMW".into()), Car(5, "Audi".into())];
    ///
    /// let l = ROIndexList::<'_, _, UIntIndex>::borrowed(|c: &Car| c.0, &cars);
    ///
    /// assert!(l.idx().contains(&2));
    /// assert!(!l.idx().contains(&99));
    /// ```
    #[inline]
    pub fn contains(&self, key: &S::Key) -> bool {
        self.0.filter.contains(key)
    }

    /// Get all items for a given `Key`.
    ///
    /// ## Example
    ///
    /// ```
    /// use fast_forward::index::{Store, uint::UIntIndex};
    /// use fast_forward::collections::ro::ROIndexList;
    ///
    /// #[derive(Debug, Eq, PartialEq, Clone)]
    /// pub struct Car(usize, String);
    ///
    /// impl Car {
    ///     fn id(&self) -> usize { self.0 }
    /// }
    ///
    /// let cars = vec![Car(2, "BMW".into()), Car(5, "Audi".into())];
    ///
    /// let l = ROIndexList::<'_, _, UIntIndex>::borrowed(Car::id, &cars);
    ///
    /// assert_eq!(Some(&Car(2, "BMW".into())), l.idx().get(&2).next());
    /// ```
    #[inline]
    pub fn get(&self, key: &S::Key) -> impl Iterator<Item = &'a <I as Index<usize>>::Output>
    where
        I: Index<usize>,
    {
        self.0.filter.0.get(key).iter().map(|i| &self.0._items[*i])
    }

    /// Combined all given `keys` with an logical `OR`.
    ///
    ///```text
    /// get_many([2, 5, 6]) => get(2) OR get(5) OR get(6)
    /// get_many(2..6]) => get(2) OR get(3) OR get(4) OR get(5)
    /// ```
    ///
    /// ## Example:
    ///
    /// ```
    /// use fast_forward::index::{Store, uint::UIntIndex};
    /// use fast_forward::collections::ro::ROIndexList;
    ///
    /// #[derive(Debug, Eq, PartialEq, Clone)]
    /// pub struct Car(usize, String);
    ///
    /// let cars = vec![
    ///     Car(2, "BMW".into()),
    ///     Car(5, "Audi".into()),
    ///     Car(2, "VW".into()),
    ///     Car(99, "Porsche".into()),
    /// ];
    ///
    /// let l = ROIndexList::<'_, _, UIntIndex>::borrowed(|c: &Car| c.0, &cars);
    ///
    /// let result = l.idx().get_many([2, 5]).collect::<Vec<_>>();
    /// assert_eq!(vec![
    ///     &Car(2, "BMW".into()),
    ///     &Car(2, "VW".into()),
    ///     &Car(5, "Audi".into()),
    ///     ],
    ///     result);
    /// ```
    #[inline]
    pub fn get_many<II>(&self, keys: II) -> impl Iterator<Item = &'a <I as Index<usize>>::Output>
    where
        II: IntoIterator<Item = S::Key> + 'a,
        I: Index<usize>,
        <I as Index<usize>>::Output: Sized,
    {
        self.0.filter.0.get_many(keys).map(|i| &self.0._items[*i])
    }

    /// Return filter methods from the `Store`.
    ///
    /// ## Example
    ///
    /// ```
    /// use fast_forward::index::{Store, uint::UIntIndex};
    /// use fast_forward::collections::ro::ROIndexList;
    ///
    /// #[derive(Debug, Eq, PartialEq, Clone)]
    /// pub struct Car(usize, String);
    ///
    /// let cars = vec![Car(2, "BMW".into()), Car(5, "Audi".into())];
    ///
    /// let l = ROIndexList::<'_, _, UIntIndex>::borrowed(|c: &Car| c.0, &cars);
    ///
    /// assert_eq!(
    ///     vec![&Car(2, "BMW".into()), &Car(5, "Audi".into())],
    ///     l.idx().filter(|fltr| fltr.eq(&2) | fltr.eq(&5)).collect::<Vec<_>>()
    /// );
    /// ```
    ///
    /// ## Hint
    ///
    /// The `OR` (`|`) generated a extra allocation.
    #[inline]
    pub fn filter<P>(&self, predicate: P) -> impl Iterator<Item = &'a <I as Index<usize>>::Output>
    where
        P: Fn(&Filter<'a, S, I>) -> Indices<'a>,
        I: Index<usize>,
    {
        predicate(&self.0).items(self.0._items)
    }

    ///
    #[inline]
    pub fn create_view<II>(&self, keys: II) -> View<'a, S, I>
    where
        II: IntoIterator<Item = S::Key> + ExactSizeIterator + 'a,
        I: Index<usize>,
    {
        View::new(keys, self.0.filter.0, self.0._items)
    }

    /// Returns Meta data, if the [`crate::index::Store`] supports any.
    #[inline]
    pub fn meta(&self) -> S::Meta<'_>
    where
        S: MetaData,
    {
        self.0.filter.0.meta()
    }
}

/// A `View` is a wrapper for an given [`Store`],
/// that can be only use (read only) for [`Filterable`] operations.
pub struct View<'a, S, I> {
    view: S,
    store: &'a S,
    items: &'a I,
}

impl<'a, S, I> View<'a, S, I>
where
    S: Store,
    I: Index<usize>,
{
    pub fn new<K>(keys: K, store: &'a S, items: &'a I) -> Self
    where
        K: IntoIterator<Item = S::Key> + ExactSizeIterator,
    {
        Self {
            view: S::from_iter(keys),
            store,
            items,
        }
    }

    pub fn contains(&self, key: &S::Key) -> bool {
        self.view.contains(key)
    }

    pub fn get(
        &'a self,
        key: &'a S::Key,
    ) -> Option<impl Iterator<Item = &'a <I as Index<usize>>::Output>> {
        if !self.view.contains(key) {
            return None;
        }

        Some(self.store.get(key).iter().map(|i| &self.items[*i]))
    }

    #[inline]
    pub fn get_many<II>(&'a self, keys: II) -> impl Iterator<Item = &'a <I as Index<usize>>::Output>
    where
        II: IntoIterator<Item = S::Key> + 'a,
        I: Index<usize>,
        <I as Index<usize>>::Output: Sized,
    {
        let keys = keys
            .into_iter()
            .filter(|key| self.contains(key))
            .collect::<Vec<_>>();
        self.store.get_many(keys).map(|i| &self.items[*i])
    }

    // pub fn items(&self) -> impl Iterator<Item = &'a <I as Index<usize>>::Output>
    // where
    //     I: IntoIterator,
    // {
    //     self.items
    //         .into_iter()
    //         .enumerate()
    //         .filter(|(i, _)| self.view.contains(i))
    // }
}
