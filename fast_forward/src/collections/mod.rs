//! Different kinds of collections which are using `Indices`.
//!
//! This collections only support one Index for one property.
//!
pub mod base;
pub mod ro;
pub mod rw;

use crate::index::{
    indices::Indices,
    store::{Filterable, MetaData, Store},
    view::{Filter, Keys, View},
    Indexable,
};

/// A `Retriever` is the main interface for get Items by an given query.
#[repr(transparent)]
pub struct Retriever<'a, S, I>(Filter<'a, S, I>);

impl<'a, S, I> Retriever<'a, S, I>
where
    S: Store,
{
    /// Create a new instance of an [`Retriever`].
    pub const fn new(store: &'a S, items: &'a I) -> Self {
        Self(Filter::new(store, items))
    }

    #[inline]
    pub fn eq(&self, key: &S::Key) -> Indices<'a, S::Index>
    where
        S::Index: Clone,
    {
        self.0.eq(key)
    }

    /// Checks whether the `Key` exists.
    ///
    /// # Example
    ///
    /// ```
    /// use fast_forward::index::{store::Store, int::IntIndex};
    /// use fast_forward::collections::ro::IList;
    ///
    /// #[derive(Debug, PartialEq)]
    /// pub struct Car(i32, String);
    ///
    /// let cars = vec![Car(-2, "BMW".into()), Car(5, "Audi".into())];
    ///
    /// let l = IList::<IntIndex, _>::new(|c| c.0, cars);
    ///
    /// assert!(l.idx().contains(&-2));
    /// assert!(!l.idx().contains(&99));
    /// ```
    #[inline]
    pub fn contains(&self, key: &S::Key) -> bool {
        self.0.filter.contains(key)
    }

    /// Get all items for a given `Key`.
    ///
    /// # Example
    ///
    /// ```
    /// use fast_forward::index::{store::Store, uint::UIntIndex};
    /// use fast_forward::collections::ro::IList;
    ///
    /// #[derive(Debug, PartialEq)]
    /// pub struct Car(usize, String);
    ///
    /// impl Car {
    ///     fn id(&self) -> usize { self.0 }
    /// }
    ///
    /// let cars = vec![Car(2, "BMW".into()), Car(5, "Audi".into())];
    ///
    /// let l = IList::<UIntIndex, _>::new(Car::id, cars);
    ///
    /// assert_eq!(Some(&Car(2, "BMW".into())), l.idx().get(&2).next());
    /// ```
    #[inline]
    pub fn get(&self, key: &S::Key) -> impl Iterator<Item = &'a <I as Indexable<S::Index>>::Output>
    where
        I: Indexable<S::Index>,
    {
        self.0.items.items(self.0.filter.get(key).iter())
    }

    /// Combined all given `keys` with an logical `OR`.
    ///
    ///```text
    /// get_many([2, 5, 6]) => get(2) OR get(5) OR get(6)
    /// get_many(2..6]) => get(2) OR get(3) OR get(4) OR get(5)
    /// ```
    ///
    /// # Example:
    ///
    /// ```
    /// use fast_forward::index::{store::Store, int::IntIndex};
    /// use fast_forward::collections::ro::IList;
    ///
    /// #[derive(Debug, PartialEq)]
    /// pub struct Car(i32, String);
    ///
    /// let cars = vec![
    ///     Car(-2, "BMW".into()),
    ///     Car(5, "Audi".into()),
    ///     Car(-2, "VW".into()),
    ///     Car(-99, "Porsche".into()),
    /// ];
    ///
    /// let l = IList::<IntIndex, _>::new(|c| c.0, cars);
    ///
    /// let result = l.idx().get_many([-2, 5]).collect::<Vec<_>>();
    /// assert_eq!(vec![
    ///     &Car(-2, "BMW".into()),
    ///     &Car(-2, "VW".into()),
    ///     &Car(5, "Audi".into()),
    ///     ],
    ///     result);
    /// ```
    #[inline]
    pub fn get_many<II>(
        &self,
        keys: II,
    ) -> impl Iterator<Item = &'a <I as Indexable<S::Index>>::Output>
    where
        II: IntoIterator<Item = S::Key> + 'a,
        I: Indexable<S::Index>,
        <I as Indexable<S::Index>>::Output: Sized,
    {
        self.0.filter.get_many(keys).items(self.0.items)
    }

    /// Return filter methods from the `Store`.
    ///
    /// # Example
    ///
    /// ```
    /// use fast_forward::index::{store::Store, uint::UIntIndex};
    /// use fast_forward::collections::ro::IList;
    ///
    /// #[derive(Debug, PartialEq)]
    /// pub struct Car(usize, String);
    ///
    /// let cars = vec![Car(2, "BMW".into()), Car(5, "Audi".into())];
    ///
    /// let l = IList::<UIntIndex, _>::new(|c| c.0, cars);
    ///
    /// assert_eq!(
    ///     vec![&Car(2, "BMW".into()), &Car(5, "Audi".into())],
    ///     l.idx().filter(|fltr| fltr.eq(&2) | fltr.eq(&5)).collect::<Vec<_>>()
    /// );
    /// ```
    ///
    /// # Hint
    ///
    /// Every `OR` (`|`) generated a extra allocation. `get_many` can be a better option.
    #[inline]
    pub fn filter<P>(
        &self,
        predicate: P,
    ) -> impl Iterator<Item = &'a <I as Indexable<S::Index>>::Output>
    where
        P: Fn(&Filter<'a, S, I>) -> Indices<'a, S::Index>,
        I: Indexable<S::Index>,
        S::Index: Clone,
    {
        predicate(&self.0).items(self.0.items)
    }

    /// Create a `View` by a given list of keys.
    /// The view represents a subset of the items in the list.
    /// This is particularly useful if I don't want to show all items for non-existing rights.
    ///
    /// # Example
    ///
    /// ```
    /// use fast_forward::index::{store::Store, int::IntIndex};
    /// use fast_forward::collections::ro::IList;
    ///
    /// #[derive(Debug, PartialEq)]
    /// pub struct Car(i32, String);
    ///
    /// let l = IList::<IntIndex, _>::new(|c| c.0, vec![
    ///                                 Car(1, "BMW".into()),
    ///                                 Car(2, "Porsche".into()),
    ///                                 Car(-3, "Mercedes".into()),
    ///                                 Car(-5, "Audi".into())]);
    ///
    /// let view = l.idx().create_view([1, 2, -3]);
    /// assert!(view.contains(&-3));
    /// assert!(view.contains(&1));
    /// assert_eq!(None, view.get(&-5).next());
    ///
    /// // or by using a `Range`
    /// let view = l.idx().create_view(-3..=3);
    /// assert!(view.contains(&-3));
    /// assert!(view.contains(&1));
    /// assert_eq!(None, view.get(&-5).next());
    /// ```
    #[inline]
    pub fn create_view<It>(&self, keys: It) -> View<'a, S, S, I>
    where
        It: IntoIterator<Item = <S as Keys>::Key>,
        S: Keys<Key = <S as Filterable>::Key>,
        I: Indexable<S::Index>,
    {
        View::new(S::from_iter(keys), self.0.filter, self.0.items)
    }

    /// Returns Meta data, if the [`crate::index::store::Store`] supports any.
    #[inline]
    pub fn meta(&self) -> S::Meta<'_>
    where
        S: MetaData,
    {
        self.0.filter.meta()
    }
}
