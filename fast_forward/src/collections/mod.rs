//! Different kinds of collections which are using `Indices`.
//!
#[doc(hidden)]
pub(crate) mod list;
pub mod ro;
pub mod rw;

use std::ops::Index;

pub use crate::collections::{ro::ROIndexList, rw::RWIndexList};

use crate::index::{self, Filterable, Indices, MetaData};

/// [`Filter`] combines a given [`Filterable`] with the given list of items.
pub struct Filter<'f, F, I> {
    filter: &'f F,
    items: &'f I,
}

impl<'f, F, I> Filter<'f, F, I>
where
    F: Filterable,
{
    const fn new(filter: &'f F, items: &'f I) -> Self {
        Self { filter, items }
    }

    #[inline]
    pub fn eq(&self, key: &F::Key) -> Indices<'f> {
        self.filter.get(key)
    }

    #[inline]
    pub fn eq_many<It>(&self, keys: It) -> Indices<'f>
    where
        It: IntoIterator<Item = F::Key>,
    {
        self.filter.get_many(keys)
    }

    #[inline]
    pub fn get(&self, i: usize) -> &<I as Index<usize>>::Output
    where
        I: Index<usize>,
    {
        &self.items[i]
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
    pub fn get(&self, key: &F::Key) -> index::Iter<'r, I>
    where
        I: Index<usize>,
    {
        self.0.eq(key).items(self.0.items)
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
    ///     &Car(5, "Audi".into()),
    ///     &Car(2, "VW".into()),
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
    pub fn get_many<II>(&self, keys: II) -> index::Iter<'r, I>
    where
        II: IntoIterator<Item = F::Key>,
        I: Index<usize>,
    {
        self.0.eq_many(keys).items(self.0.items)
    }

    /// Combined all given `keys` with an logical `OR`.
    /// The result is getting per callback function with the args:
    /// `key` and an Iterator over all filtering Items.
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
    /// let cars = vec![
    ///     Car(2, "BMW".into()),
    ///     Car(5, "Audi".into()),
    ///     Car(2, "VW".into()),
    ///     Car(99, "Porsche".into()),
    /// ];
    ///
    /// let l = ROIndexList::<'_, _, UIntIndex>::borrowed(|c: &Car| c.0, &cars);
    ///
    /// l.idx().get_many_cb([2, 5], |k, items| {
    ///     let l = items.collect::<Vec<_>>();
    ///     match k {
    ///         2 => assert_eq!(vec![&Car(2, "BMW".into()), &Car(2, "VW".into())], l),
    ///         5 => assert_eq!(vec![&Car(5, "Audi".into())], l),
    ///         _ => unreachable!("invalid Key: {k}"),
    ///     }
    /// });
    /// ```
    #[inline]
    pub fn get_many_cb<II, C>(&self, keys: II, callback: C)
    where
        II: IntoIterator<Item = F::Key>,
        I: Index<usize>,
        C: Fn(&F::Key, index::Iter<'r, I>),
    {
        for k in keys {
            callback(&k, self.0.eq(&k).items(self.0.items))
        }
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
        !self.0.eq(key).is_empty()
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
    pub fn filter<P>(&self, predicate: P) -> index::Iter<'r, I>
    where
        P: Fn(&Filter<'r, F, I>) -> Indices<'r>,
        I: Index<usize>,
    {
        predicate(&self.0).items(self.0.items)
    }

    /// Returns Meta data, if the [`crate::index::Store`] supports any.
    #[inline]
    pub fn meta(&self) -> F::Meta<'_>
    where
        F: MetaData,
    {
        self.0.filter.meta()
    }
}
