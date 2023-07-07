//! The Idea of a `View` is like by databases.
//! Show a section of an [`crate::index::store::Store`].

use std::ops::Index;

use super::{indices::Indices, store::Filterable};

/// [`Filter`] combines a given [`Filterable`] with the given list of items.
pub struct Filter<'a, F, I> {
    pub(crate) filter: &'a F,
    pub(crate) items: &'a I,
}

impl<'a, F, I> Filter<'a, F, I>
where
    F: Filterable,
{
    pub const fn new(filter: &'a F, items: &'a I) -> Self {
        Self { filter, items }
    }

    #[inline]
    pub fn eq(&self, key: &F::Key) -> Indices<'a, F::Index>
    where
        F::Index: Clone,
    {
        Indices::from_sorted_slice(self.filter.get(key))
    }

    #[inline]
    pub fn contains(&self, key: &F::Key) -> bool {
        self.filter.contains(key)
    }

    #[inline]
    pub fn items(&'a self, key: &F::Key) -> impl Iterator<Item = &'a <I as Index<F::Index>>::Output>
    where
        I: Index<F::Index>,
        F::Index: Clone,
    {
        self.filter.get(key).iter().map(|i| &self.items[i.clone()])
    }
}

/// [`Keys`] is a special kind of a `Store`, which stores only `Key`s.
/// This is useful, if you want to create a `View` of a [`Store`].
pub trait Keys {
    type Key;

    /// Checks if the `Key`exist.
    fn exist(&self, key: &Self::Key) -> bool;

    /// Insert a new `Key`. If the Key already exists, then will be ignored.
    fn add_key(&mut self, key: Self::Key);

    /// Create a new `Key-Store` from a given List of `Key`s.
    fn from_iter<I>(it: I) -> Self
    where
        I: IntoIterator<Item = Self::Key>;
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
    K: Keys<Key = F::Key>,
    F: Filterable,
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

    #[derive(Debug, PartialEq)]
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
