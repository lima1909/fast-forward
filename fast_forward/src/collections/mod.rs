//! Different kinds of collections which are using `Indices`.
//!
#[doc(hidden)]
pub(crate) mod list;
pub mod ro;
pub mod rw;

use std::ops::Index;

pub use crate::collections::{ro::ROIndexList, rw::RWIndexList};

use crate::index::{self, store::Many, Filterable, Indices, Iter, MetaData};

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
        self.filter.get(key)
    }

    #[inline]
    pub fn items(&self, key: &F::Key) -> index::Iter<'f, I>
    where
        I: Index<usize>,
    {
        self.filter.get(key).items(self._items)
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
        self.0.filter.get(key)
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
    pub fn get(&self, key: &F::Key) -> index::Iter<'r, I>
    where
        I: Index<usize>,
    {
        self.0.items(key)
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
    pub fn get_many<II>(&self, keys: II) -> Many<'r, <II as IntoIterator>::IntoIter, F, I>
    where
        II: IntoIterator<Item = F::Key>,
        I: Index<usize>,
    {
        Many::new(keys.into_iter(), self.0.filter, self.0._items)
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
            callback(&k, self.0.items(&k))
        }
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
        predicate(&self.0).items(self.0._items)
    }

    pub fn create_view<II>(&'r self, keys: II) -> View<'r, II::IntoIter, F, I>
    where
        II: IntoIterator<Item = F::Key>,
        I: Index<usize>,
    {
        View::new(keys.into_iter(), &self.0)
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

// ---------------------------------------------------------
pub struct View<'v, Keys, Fltr, Items: Index<usize>> {
    keys: Keys,
    filter: &'v Filter<'v, Fltr, Items>,
    iter: Iter<'v, Items>,
}

impl<'v, Keys, Fltr, Items> View<'v, Keys, Fltr, Items>
where
    Items: Index<usize>,
    Fltr: Filterable,
{
    pub fn new(mut keys: Keys, filter: &'v Filter<'v, Fltr, Items>) -> Self
    where
        Keys: Iterator<Item = Fltr::Key>,
    {
        Self {
            iter: match keys.next() {
                Some(k) => filter.filter.get(&k),
                None => Indices::empty(),
            }
            .items(filter._items),
            keys,
            filter,
        }
    }

    #[inline]
    pub fn contains(&self, key: &Fltr::Key) -> bool {
        !self.filter.eq(key).is_empty()
    }
}

impl<'v, Keys, Fltr, Items> Iterator for View<'v, Keys, Fltr, Items>
where
    Fltr: Filterable,
    Keys: Iterator<Item = Fltr::Key>,
    Items: Index<usize>,
    <Items as Index<usize>>::Output: Sized,
{
    type Item = &'v Items::Output;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(i) = self.iter.next() {
            return Some(i);
        }

        let mut key = self.keys.next()?;
        let mut idx = self.filter.eq(&key);

        while idx.is_empty() {
            key = self.keys.next()?;
            idx = self.filter.eq(&key);
        }

        self.iter = idx.items(self.filter._items);
        self.iter.next()
    }
}

#[cfg(test)]
mod tests {
    use crate::index::{map::MapIndex, Store};

    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::empty(vec![], vec![])]
    #[case::one_found(vec!["c"], vec![&"c"])]
    #[case::one_not_found(vec!["-"], vec![])]
    #[case::m_z_a(vec!["m", "z", "a"], vec![&"z", &"a"])]
    #[case::a_m_z(vec![ "a","m", "z"], vec![&"a", &"z"])]
    #[case::z_m_a(vec![ "z","m", "a"], vec![&"z", &"a"])]
    #[case::m_z_a_m(vec!["m", "z", "a", "m"], vec![&"z", &"a"])]
    #[case::m_z_a_m_m(vec!["m", "z", "a", "m", "m"], vec![&"z", &"a"])]
    #[case::double(vec!["x"], vec![&"x",&"x"])]
    fn view_str(#[case] keys: Vec<&str>, #[case] expected: Vec<&&str>) {
        let items = vec!["x", "a", "b", "c", "x", "y", "z"];
        let map = MapIndex::from_iter(items.clone().into_iter());
        let filter = Filter::new(&map, &items);
        let result = View::new(keys.into_iter(), &filter).collect::<Vec<_>>();

        assert_eq!(expected, result);
    }
}
