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
        self.filter.0.contains(key)
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
    pub const fn new(store: &'a S, items: &'a I) -> Self {
        Self(Filter::new(store, items))
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
        View::new(S::from_iter(keys), self.0.filter.0, self.0._items)
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
pub struct View<'a, F, I> {
    view: F,
    store: &'a F,
    items: &'a I,
}

impl<'a, F, I> View<'a, F, I>
where
    F: Filterable,
    I: Index<usize>,
{
    pub fn new(view: F, store: &'a F, items: &'a I) -> Self {
        Self { view, store, items }
    }

    #[inline]
    pub fn eq(&self, key: &F::Key) -> Indices<'a> {
        Indices::from_sorted_slice(self.store.get_with_check(key, |k| self.view.contains(k)))
    }

    #[inline]
    pub fn contains(&self, key: &F::Key) -> bool {
        self.view.contains(key)
    }

    #[inline]
    pub fn get(&'a self, key: &'a F::Key) -> impl Iterator<Item = &'a <I as Index<usize>>::Output> {
        self.store
            .get_with_check(key, |k| self.view.contains(k))
            .iter()
            .map(|i| &self.items[*i])
    }

    #[inline]
    pub fn get_many<II>(&'a self, keys: II) -> impl Iterator<Item = &'a <I as Index<usize>>::Output>
    where
        II: IntoIterator<Item = F::Key> + 'a,
        I: Index<usize>,
        <I as Index<usize>>::Output: Sized,
    {
        let keys = keys.into_iter().filter(|key| self.view.contains(key));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::uint::UIntIndex;
    use rstest::{fixture, rstest};

    #[derive(Debug, Clone, PartialEq)]
    pub struct Car {
        id: usize,
        name: &'static str,
    }

    #[fixture]
    fn list<'a>() -> ROIndexList<'a, Car, UIntIndex> {
        ROIndexList::owned(
            |c| c.id,
            vec![
                Car {
                    id: 99,
                    name: "BMW 1",
                },
                Car {
                    id: 7,
                    name: "Audi",
                },
                Car {
                    id: 99,
                    name: "BMW 2",
                },
                Car {
                    id: 1,
                    name: "Porsche",
                },
            ],
        )
    }

    #[rstest]
    fn view_without_7(list: ROIndexList<'_, Car, UIntIndex>) {
        let view = list.idx().create_view(vec![1, 3, 99].into_iter());

        assert!(!view.contains(&7));
        assert_eq!(None, view.get(&7).next());
        assert!(view.get_many([7]).next().is_none());
    }

    #[rstest]
    fn view_eq(list: ROIndexList<'_, Car, UIntIndex>) {
        let view = list.idx().create_view(vec![1, 3, 99].into_iter());

        assert!(view.eq(&7).as_slice().iter().next().is_none());
        assert!(view.eq(&2000).as_slice().iter().next().is_none());

        assert_eq!([3], view.eq(&1));
        assert_eq!([0, 2, 3], view.eq(&1) | view.eq(&99));
    }

    #[rstest]
    fn view_get_without_7(list: ROIndexList<'_, Car, UIntIndex>) {
        let view = list.idx().create_view(vec![1, 3, 99].into_iter());

        assert_eq!(3, view.get_many([1, 99, 7]).collect::<Vec<_>>().len());

        let mut it = view.get(&99);
        assert_eq!(
            Some(&Car {
                id: 99,
                name: "BMW 1",
            }),
            it.next()
        );
        assert_eq!(
            Some(&Car {
                id: 99,
                name: "BMW 2",
            }),
            it.next()
        );
        assert_eq!(None, it.next());
    }

    #[rstest]
    fn view_get_many_without_7(list: ROIndexList<'_, Car, UIntIndex>) {
        let view = list.idx().create_view(vec![1, 3, 99].into_iter());

        let mut it = view.get_many([99, 7]);
        assert_eq!(
            Some(&Car {
                id: 99,
                name: "BMW 1",
            }),
            it.next()
        );
        assert_eq!(
            Some(&Car {
                id: 99,
                name: "BMW 2",
            }),
            it.next()
        );
        assert_eq!(None, it.next());
    }

    #[rstest]
    fn view_with_range(list: ROIndexList<'_, Car, UIntIndex>) {
        assert!(!list.idx().create_view(10..100).contains(&7))
    }
}
