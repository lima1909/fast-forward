//! Different kinds of collections which are using `Indices`.
//!
#[doc(hidden)]
pub(crate) mod list;
pub mod ro;
pub mod rw;

use std::ops::Index;

use crate::index::{
    indices::Indices,
    store::{Filter as StoreFilter, Filterable, Keys, MetaData, Store},
};

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
    pub fn eq(&self, key: &F::Key) -> Indices<'a, F::Index>
    where
        F::Index: Clone,
    {
        self.filter.eq(key)
    }

    #[inline]
    pub fn contains(&self, key: &F::Key) -> bool {
        self.filter.0.contains(key)
    }

    #[inline]
    pub fn items(&'a self, key: &F::Key) -> impl Iterator<Item = &'a <I as Index<F::Index>>::Output>
    where
        I: Index<F::Index>,
        F::Index: Clone,
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
    pub fn eq(&self, key: &S::Key) -> Indices<'a, S::Index>
    where
        S::Index: Clone,
    {
        self.0.eq(key)
    }

    /// Checks whether the `Key` exists.
    ///
    /// ## Example
    ///
    /// ```
    /// use fast_forward::index::{store::Store, uint::UIntIndex};
    /// use fast_forward::collections::ro::IList;
    ///
    /// #[derive(Debug, Eq, PartialEq, Clone)]
    /// pub struct Car(usize, String);
    ///
    /// let cars = vec![Car(2, "BMW".into()), Car(5, "Audi".into())];
    ///
    /// let l = IList::<UIntIndex, _>::new(|c| c.0, cars);
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
    /// use fast_forward::index::{store::Store, uint::UIntIndex};
    /// use fast_forward::collections::ro::IList;
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
    /// let l = IList::<UIntIndex, _>::new(Car::id, cars);
    ///
    /// assert_eq!(Some(&Car(2, "BMW".into())), l.idx().get(&2).next());
    /// ```
    #[inline]
    pub fn get(&self, key: &S::Key) -> impl Iterator<Item = &'a <I as Index<S::Index>>::Output>
    where
        I: Index<S::Index>,
        S::Index: Clone,
    {
        self.0
            .filter
            .0
            .get(key)
            .iter()
            .map(|i| &self.0._items[i.clone()])
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
    /// use fast_forward::index::{store::Store, uint::UIntIndex};
    /// use fast_forward::collections::ro::IList;
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
    /// let l = IList::<UIntIndex, _>::new(|c| c.0, cars);
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
    pub fn get_many<II>(&self, keys: II) -> impl Iterator<Item = &'a <I as Index<S::Index>>::Output>
    where
        II: IntoIterator<Item = S::Key> + 'a,
        I: Index<S::Index>,
        <I as Index<S::Index>>::Output: Sized,
        S::Index: Clone,
    {
        self.0
            .filter
            .0
            .get_many(keys)
            .map(|i| &self.0._items[i.clone()])
    }

    /// Return filter methods from the `Store`.
    ///
    /// ## Example
    ///
    /// ```
    /// use fast_forward::index::{store::Store, uint::UIntIndex};
    /// use fast_forward::collections::ro::IList;
    ///
    /// #[derive(Debug, Eq, PartialEq, Clone)]
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
    /// ## Hint
    ///
    /// The `OR` (`|`) generated a extra allocation.
    #[inline]
    pub fn filter<P>(
        &self,
        predicate: P,
    ) -> impl Iterator<Item = &'a <I as Index<S::Index>>::Output>
    where
        P: Fn(&Filter<'a, S, I>) -> Indices<'a, S::Index>,
        I: Index<S::Index>,
        S::Index: Clone,
    {
        predicate(&self.0).items(self.0._items)
    }

    ///
    #[inline]
    pub fn create_view<It>(&self, keys: It) -> View<'a, S, S, I>
    where
        It: IntoIterator<Item = <S as Keys>::Key>,
        I: Index<S::Index>,
        S: Filterable,
        S: Keys<Key = <S as Filterable>::Key>,
    {
        View::new(S::from_iter(keys), self.0.filter.0, self.0._items)
    }

    /// Returns Meta data, if the [`crate::index::store::Store`] supports any.
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
pub struct View<'a, K, F, I> {
    keys: K,
    store: &'a F,
    items: &'a I,
}

impl<'a, K, F, I> View<'a, K, F, I>
where
    F: Filterable,
    K: Keys<Key = F::Key>,
    I: Index<F::Index>,
{
    pub fn new(keys: K, store: &'a F, items: &'a I) -> Self {
        Self { keys, store, items }
    }

    #[inline]
    pub fn eq(&self, key: &K::Key) -> Indices<'a, F::Index>
    where
        F::Index: Clone,
    {
        Indices::from_sorted_slice(self.store.get_with_check(key, |k| self.keys.exist(k)))
    }

    #[inline]
    pub fn contains(&self, key: &K::Key) -> bool {
        self.keys.exist(key)
    }

    #[inline]
    pub fn get(
        &'a self,
        key: &'a F::Key,
    ) -> impl Iterator<Item = &'a <I as Index<F::Index>>::Output>
    where
        F::Index: Clone,
    {
        self.store
            .get_with_check(key, |k| self.keys.exist(k))
            .iter()
            .map(|i| &self.items[i.clone()])
    }

    #[inline]
    pub fn get_many<II>(
        &'a self,
        keys: II,
    ) -> impl Iterator<Item = &'a <I as Index<F::Index>>::Output>
    where
        II: IntoIterator<Item = F::Key> + 'a,
        I: Index<F::Index>,
        <I as Index<F::Index>>::Output: Sized,
        F::Index: Clone,
    {
        let keys = keys.into_iter().filter(|key| self.keys.exist(key));
        self.store.get_many(keys).map(|i| &self.items[i.clone()])
    }

    #[inline]
    pub fn filter<P>(
        &'a self,
        predicate: P,
    ) -> impl Iterator<Item = &'a <I as Index<usize>>::Output>
    where
        P: Fn(Filter<'a, View<'a, K, F, I>, I>) -> Indices<'a>,
        I: Index<usize>,
        F::Index: Clone,
    {
        let filter = Filter::new(self, self.items);
        predicate(filter).items(self.items)
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

impl<'a, K, F, I> Filterable for View<'a, K, F, I>
where
    K: Keys,
    F: Filterable<Key = K::Key>,
{
    type Key = F::Key;
    type Index = F::Index;

    fn contains(&self, key: &Self::Key) -> bool {
        self.keys.exist(key)
    }

    fn get(&self, key: &Self::Key) -> &[F::Index] {
        self.store.get_with_check(key, |k| self.keys.exist(k))
    }
}

#[cfg(test)]
mod tests {
    use crate::collections::ro::IList;
    use crate::index::uint::UIntIndex;
    use rstest::{fixture, rstest};

    #[derive(Debug, Clone, PartialEq)]
    pub struct Car {
        id: usize,
        name: &'static str,
    }

    #[fixture]
    fn list<'a>() -> IList<UIntIndex, Car> {
        IList::new(
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
    fn view_eq(list: IList<UIntIndex, Car>) {
        let view = list.idx().create_view([1, 3, 99]);

        assert!(view.eq(&7).as_slice().iter().next().is_none());
        assert!(view.eq(&2000).as_slice().iter().next().is_none());

        assert_eq!([3], view.eq(&1));
        assert_eq!([0, 2, 3], view.eq(&1) | view.eq(&99));
    }

    #[rstest]
    fn view_filter(list: IList<UIntIndex, Car>) {
        let view = list.idx().create_view([1, 3, 99]);

        // 7 is not allowed
        assert_eq!(None, view.filter(|f| f.eq(&7)).next());
        // 2000 do not exist
        assert_eq!(None, view.filter(|f| f.eq(&2000)).next());

        assert_eq!(
            vec![&Car {
                id: 1,
                name: "Porsche",
            }],
            view.filter(|f| f.eq(&1)).collect::<Vec<_>>()
        );

        assert_eq!(
            vec![
                &Car {
                    id: 99,
                    name: "BMW 1",
                },
                &Car {
                    id: 99,
                    name: "BMW 2",
                },
                &Car {
                    id: 1,
                    name: "Porsche",
                },
            ],
            view.filter(|f| f.eq(&1) | f.eq(&99)).collect::<Vec<_>>()
        );

        assert_eq!(
            vec![
                &Car {
                    id: 99,
                    name: "BMW 1",
                },
                &Car {
                    id: 99,
                    name: "BMW 2",
                },
            ],
            view.filter(|f| {
                assert_eq!(
                    Some(&Car {
                        id: 1,
                        name: "Porsche",
                    }),
                    f.items(&1).next()
                );

                // 7 is not allowed
                assert!(f.items(&7).next().is_none());

                assert!(!f.contains(&7));
                assert!(f.contains(&1));

                f.eq(&99)
            })
            .collect::<Vec<_>>()
        );
    }

    #[rstest]
    fn view_without_7(list: IList<UIntIndex, Car>) {
        let view = list.idx().create_view([1, 3, 99]);

        assert!(!view.contains(&7));
        assert_eq!(None, view.get(&7).next());
        assert!(view.get_many([7]).next().is_none());
    }

    #[rstest]
    fn view_get_without_7(list: IList<UIntIndex, Car>) {
        let view = list.idx().create_view([1, 3, 99]);

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
    fn view_get_many_without_7(list: IList<UIntIndex, Car>) {
        let view = list.idx().create_view([1, 3, 99]);

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
    fn view_with_range(list: IList<UIntIndex, Car>) {
        assert!(!list.idx().create_view(10..100).contains(&7))
    }
}
