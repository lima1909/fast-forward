//! Base-List for indexed read-write lists.
//!
use std::{fmt::Debug, ops::Deref};

use crate::{
    collections::{rw::Editable, Retriever},
    index::store::{Filterable, Store},
};

/// Is a Wrapper for an [`Vec`], which has trigger functions for insert and remove operations
#[repr(transparent)]
#[derive(Debug)]
pub struct TriggerList<I>(Vec<I>);

impl<I> TriggerList<I> {
    /// Create a `List` with given `capacity`.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    // Return the `Item` from the given index for updating the `Item`.
    #[inline]
    pub fn get_mut(&mut self, pos: usize) -> Option<&mut I> {
        self.0.get_mut(pos)
    }

    /// Append a new `Item` to the List.
    #[inline]
    pub fn push<Trigger>(&mut self, item: I, mut insert: Trigger) -> usize
    where
        Trigger: FnMut(&I, usize),
    {
        let idx = self.0.len();
        insert(&item, idx);
        self.0.push(item);
        idx
    }

    /// The Item in the list will be removed.
    #[inline]
    pub fn remove<Trigger>(&mut self, pos: usize, mut trigger: Trigger) -> Option<I>
    where
        Trigger: FnMut(StoreOp, &I, usize),
    {
        if self.0.is_empty() {
            return None;
        }

        let last_idx = self.0.len() - 1;
        // index out of bound
        if pos > last_idx {
            return None;
        }

        // last item in the list
        if pos == last_idx {
            let rm_item = self.0.remove(pos);
            trigger(StoreOp::Delete, &rm_item, pos);
            return Some(rm_item);
        }

        // remove item and entry in store and swap with last item
        let rm_item = self.0.swap_remove(pos);
        trigger(StoreOp::Delete, &rm_item, pos);

        // formerly last item, now item on pos
        let curr_item = &self.0[pos];
        trigger(StoreOp::Delete, curr_item, last_idx); // remove formerly entry in store
        trigger(StoreOp::Insert, curr_item, pos);

        Some(rm_item)
    }
}

pub enum StoreOp {
    Delete,
    Insert,
}

impl<I> Deref for TriggerList<I> {
    type Target = Vec<I>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<I> Default for TriggerList<I> {
    fn default() -> Self {
        Self(Default::default())
    }
}

///
/// `List` is a list with one `Store`.
/// This means, one `Index`.
///
#[derive(Debug)]
pub struct List<S, I, F> {
    field: F,
    store: S,
    items: TriggerList<I>,
}

impl<S, I, F> List<S, I, F>
where
    S: Store<Index = usize>,
    F: Fn(&I) -> S::Key,
{
    pub fn new(field: F) -> Self {
        Self {
            field,
            store: S::with_capacity(0),
            items: TriggerList::with_capacity(0),
        }
    }

    pub fn from_vec(field: F, v: Vec<I>) -> Self {
        #[allow(clippy::useless_conversion)]
        // call into_iter is is necessary, because Vec not impl: ExactSizeIterator
        Self::from_iter(field, v.into_iter())
    }

    pub fn from_iter<It>(field: F, iter: It) -> Self
    where
        It: IntoIterator<Item = I> + ExactSizeIterator,
    {
        let mut s = Self {
            field,
            store: S::with_capacity(iter.len()),
            items: TriggerList::with_capacity(iter.len()),
        };

        iter.into_iter().for_each(|item| {
            s.push(item);
        });

        s
    }

    pub fn idx(&self) -> Retriever<'_, S, Vec<I>> {
        Retriever::new(&self.store, &self.items)
    }
}

impl<S, I, F> Editable<I> for List<S, I, F>
where
    S: Store<Index = usize>,
    F: Fn(&I) -> S::Key,
{
    /// Append a new `Item` to the List.
    fn push(&mut self, item: I) -> usize {
        self.items.push(item, |i, idx| {
            self.store.insert((self.field)(i), idx);
        })
    }

    /// Update the item on the given position.
    fn update<U>(&mut self, pos: usize, mut update: U) -> Option<&I>
    where
        U: FnMut(&mut I),
    {
        self.items.get_mut(pos).map(|item| {
            let key = (self.field)(item);
            update(item);
            self.store.update(key, pos, (self.field)(item));
            &*item
        })
    }

    /// The Item in the list will be removed.
    ///
    /// ## Hint:
    /// The remove is a swap_remove ([`std::vec::Vec::swap_remove`])
    fn remove(&mut self, pos: usize) -> Option<I> {
        self.items.remove(pos, |trigger, i, idx| match trigger {
            StoreOp::Delete => self.store.delete((self.field)(i), &idx),
            StoreOp::Insert => self.store.insert((self.field)(i), idx),
        })
    }
}

impl<S, I, F> Filterable for List<S, I, F>
where
    S: Store<Index = usize>,
{
    type Key = S::Key;
    type Index = S::Index;

    fn contains(&self, key: &Self::Key) -> bool {
        self.store.contains(key)
    }

    fn get(&self, key: &Self::Key) -> &[Self::Index] {
        self.store.get(key)
    }
}

impl<S, I, F> Deref for List<S, I, F> {
    type Target = Vec<I>;

    fn deref(&self) -> &Self::Target {
        &self.items.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::IntIndex;
    use rstest::{fixture, rstest};

    impl<T> From<Vec<T>> for TriggerList<T> {
        fn from(v: Vec<T>) -> Self {
            Self(v)
        }
    }

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
    pub fn v() -> TriggerList<String> {
        let v: TriggerList<String> =
            vec![String::from("A"), String::from("B"), String::from("C")].into();
        v
    }

    #[test]
    fn check_methods() {
        let mut l = TriggerList::default();
        assert_eq!(
            0,
            l.push("A", |i, x| {
                assert_eq!(&"A", i);
                assert_eq!(0, x);
            })
        );

        let i = l.get_mut(0).unwrap();
        *i = "B"; // update
        assert_eq!(&"B", i);
        assert_eq!(&"B", l.first().unwrap());

        assert_eq!(1, l.len());

        let i = l.remove(0, |_, _, _| {});
        assert_eq!("B", i.unwrap());
        assert_eq!(0, l.len());
    }

    #[rstest]
    fn insert_trigger(mut v: TriggerList<String>) {
        let mut call_trigger_pos = 0usize;
        assert_eq!(
            3,
            v.push(String::from("D"), |_, pos| {
                call_trigger_pos += pos;
            })
        );
        assert_eq!(3, call_trigger_pos);
    }

    #[rstest]
    fn update(mut v: TriggerList<String>) {
        assert_eq!(Some(&String::from("A")), v.get(0));

        // update: "A" -> "AA" => (1, 2)
        let s = v.get_mut(0).unwrap();
        *s = String::from("AA");
        assert_eq!(Some(&String::from("AA")), v.get(0));
    }

    #[rstest]
    fn update_not_found(mut v: TriggerList<String>) {
        assert!(v.get_mut(10_000).is_none());
    }

    #[rstest]
    fn update_deleted_item(mut v: TriggerList<String>) {
        assert_eq!(&"A", &v[0]);
        v.remove(0, |_, _, _| {});
        assert_eq!(&"C", &v[0]);
    }

    #[rstest]
    fn remove_no_trigger(mut v: TriggerList<String>) {
        let mut call_trigger_pos = 0usize;
        v.remove(1000, |_, _, _| {
            call_trigger_pos += 1000;
        });
        assert_eq!(0, call_trigger_pos);
    }

    #[rstest]
    fn remove_first(mut v: TriggerList<String>) {
        assert_eq!(String::from("A"), v.remove(0, |_, _, _| {}).unwrap());

        assert_eq!(2, v.len());
        assert!(!v.is_empty());
        assert_eq!(&String::from("C"), v.get(0).unwrap());

        let mut it = v.iter();
        assert_eq!(Some(&"C".into()), it.next());
        assert_eq!(Some(&"B".into()), it.next());
        assert_eq!(None, it.next());
    }

    #[rstest]
    fn drop_mid(mut v: TriggerList<String>) {
        assert_eq!(String::from("B"), v.remove(1, |_, _, _| {}).unwrap());

        assert_eq!(2, v.len());
        assert!(!v.is_empty());
        assert_eq!(&String::from("C"), v.get(1).unwrap());

        let mut it = v.iter();
        assert_eq!(Some(&"A".into()), it.next());
        assert_eq!(Some(&"C".into()), it.next());
        assert_eq!(None, it.next());
    }

    #[rstest]
    fn drop_last(mut v: TriggerList<String>) {
        assert_eq!(String::from("C"), v.remove(2, |_, _, _| {}).unwrap());

        assert_eq!(2, v.len());
        assert_eq!(None, v.get(2));

        let mut it = v.iter();
        assert_eq!(Some(&"A".into()), it.next());
        assert_eq!(Some(&"B".into()), it.next());
        assert_eq!(None, it.next());
    }

    #[rstest]
    fn delete_bad_index(mut v: TriggerList<String>) {
        assert_eq!(None, v.remove(1000, |_, _, _| {}));
    }

    fn check_key_idx<S, I, F>(l: &mut List<S, I, F>)
    where
        S: Store<Index = usize>,
        F: Fn(&I) -> S::Key,
    {
        l.items.iter().enumerate().for_each(|(pos, item)| {
            let key = (l.field)(item);
            assert_eq!([pos], l.store.get(&key));
        });
    }

    #[test]
    fn check_key_idx_intindex() {
        let v = vec![
            Person::new(0, "Paul"),
            Person::new(-2, "Mario"),
            Person::new(2, "Jasmin"),
        ];
        check_key_idx(&mut List::<IntIndex, Person, _>::from_iter(
            |p| p.id,
            v.iter().cloned(),
        ));

        let mut l = List::<IntIndex, Person, _>::from_iter(|p| p.id, v.iter().cloned());
        l.remove(0);
        check_key_idx(&mut l);

        let mut l = List::<IntIndex, Person, _>::from_iter(|p| p.id, v.iter().cloned());
        l.remove(1);
        check_key_idx(&mut l);

        let mut l = List::<IntIndex, Person, _>::from_iter(|p| p.id, v.iter().cloned());
        l.remove(2);
        check_key_idx(&mut l);

        let mut l = List::<IntIndex, Person, _>::from_iter(|p| p.id, v.iter().cloned());
        l.remove(100);
        check_key_idx(&mut l);

        let mut l = List::<IntIndex, Person, _>::from_iter(|p| p.id, v.iter().cloned());
        l.remove(0);
        check_key_idx(&mut l);
        l.remove(0);
        check_key_idx(&mut l);
        l.remove(0);
        check_key_idx(&mut l);
        l.remove(0);
        check_key_idx(&mut l);

        let mut l = List::<IntIndex, Person, _>::from_iter(|p| p.id, v.iter().cloned());
        l.remove(1);
        check_key_idx(&mut l);
        l.remove(1);
        check_key_idx(&mut l);
        l.remove(1);
        check_key_idx(&mut l);
        l.remove(0);
        check_key_idx(&mut l);
        assert_eq!(0, l.len());
    }

    #[test]
    fn check_key_with_many_idx_intindex() {
        let v = vec![
            Person::new(-2, "Paul"),
            Person::new(-2, "Mario"),
            Person::new(2, "Jasmin"),
        ];

        let mut l = List::<IntIndex, Person, _>::from_iter(|p| p.id, v.iter().cloned());
        l.remove(0);
        check_key_idx(&mut l);

        let mut l = List::<IntIndex, Person, _>::from_iter(|p| p.id, v.iter().cloned());
        l.remove(1);
        check_key_idx(&mut l);
    }
}
