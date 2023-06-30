//! Different kinds of collections which are using `Indices`.
//!
#[doc(hidden)]
pub(crate) mod list;
pub mod ro;
pub mod rw;

use std::ops::Index;

pub use crate::collections::{ro::ROIndexList, rw::RWIndexList};

use crate::index::{indices, Filterable, Indices, MetaData};

/// [`Filter`] combines a given [`Filterable`] with the given list of items.
pub struct Filter<'f, F, I> {
    filter: &'f F,
    _items: &'f I,
}

impl<'f, F, I> Filter<'f, F, I>
where
    F: Filterable,
{
    const fn new(filter: &'f F, items: &'f I) -> Self {
        Self {
            filter,
            _items: items,
        }
    }

    #[inline]
    pub fn eq(&self, key: &F::Key) -> Indices<'f> {
        self.filter.get(key).into()
    }

    #[inline]
    pub fn items(&self, key: &F::Key) -> impl Iterator<Item = &'f <I as Index<usize>>::Output>
    where
        I: Index<usize>,
    {
        self.filter.get(key).iter().map(|i| &self._items[*i])
    }
}

/// A `Retriever` is the interface for get Items by an given filter|query.
#[repr(transparent)]
pub struct Retriever<'r, F, I>(Filter<'r, F, I>);

impl<'r, F, I> Retriever<'r, F, I>
where
    F: Filterable,
{
    /// Create a new instance of an [`Retriever`].
    pub const fn new(filter: &'r F, items: &'r I) -> Self {
        Self(Filter::new(filter, items))
    }

    #[inline]
    pub fn eq(&self, key: &F::Key) -> Indices<'r> {
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
    pub fn contains(&self, key: &F::Key) -> bool {
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
    pub fn get(&self, key: &F::Key) -> impl Iterator<Item = &'r <I as Index<usize>>::Output>
    where
        I: Index<usize>,
    {
        self.0.filter.get(key).iter().map(|i| &self.0._items[*i])
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
    ///
    /// ## Hint:
    ///
    /// The `OR` generated a extra allocation.
    ///
    /// For performance reason it is better to use [`Self::get_many_cb()`] or
    /// to call [`Self::get()`] several times.
    #[inline]
    pub fn get_many<II>(&self, keys: II) -> impl Iterator<Item = &'r <I as Index<usize>>::Output>
    where
        II: IntoIterator<Item = F::Key> + 'r,
        I: Index<usize>,
        <I as Index<usize>>::Output: Sized,
    {
        self.0.filter.get_many(keys).map(|i| &self.0._items[*i])
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
    pub fn filter<P>(&self, predicate: P) -> indices::Iter<'r, I>
    where
        P: Fn(&Filter<'r, F, I>) -> Indices<'r>,
        I: Index<usize>,
    {
        predicate(&self.0).items(self.0._items)
    }

    // ???
    // #[inline]
    // pub fn filter<P>(
    //     &self,
    //     predicate: P,
    // ) -> impl Iterator<Item = &'r <I as Index<usize>>::Output> + '_
    // where
    //     P: Fn(&Filter<'r, F, I>) -> Indices<'r>,
    //     I: Index<usize>,
    // {
    //     predicate(&self.0).items(|i| &self.0._items[i])
    // }

    /// Returns Meta data, if the [`crate::index::Store`] supports any.
    #[inline]
    pub fn meta(&self) -> F::Meta<'_>
    where
        F: MetaData,
    {
        self.0.filter.meta()
    }
}
