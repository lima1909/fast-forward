use std::ops::Deref;

use crate::{collections::Retriever, index::store::Store};

pub struct IList<S, I, F> {
    store: S,
    field: F,
    items: Vec<I>,
}

impl<S, I, F> IList<S, I, F>
where
    S: Store<Index = usize>,
    F: Fn(&I) -> S::Key,
{
    pub fn new(field: F) -> Self {
        Self {
            store: S::with_capacity(0),
            field,
            items: vec![],
        }
    }

    pub fn from_iter<It>(field: F, iter: It) -> Self
    where
        It: IntoIterator<Item = I> + ExactSizeIterator,
    {
        let mut s = Self {
            store: S::with_capacity(iter.len()),
            field,
            items: Vec::with_capacity(iter.len()),
        };

        iter.into_iter().for_each(|item| {
            s.push(item);
        });

        s
    }

    /// Append a new `Item` to the List.
    pub fn push(&mut self, item: I) {
        let idx = self.items.len();
        self.store.insert((self.field)(&item), idx);
        self.items.push(item);
    }

    /// Update the item on the given position.
    pub fn update<U>(&mut self, pos: usize, mut update: U) -> Option<&I>
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
    pub fn remove(&mut self, pos: usize) -> Option<I> {
        if self.items.is_empty() {
            return None;
        }

        let last_idx = self.items.len() - 1;
        if pos > last_idx {
            return None;
        }

        if last_idx == pos {
            let rm_item = self.items.remove(pos);
            self.store.delete((self.field)(&rm_item), &pos);
            return Some(rm_item);
        }

        let rm_item = self.items.swap_remove(pos);
        self.store.delete((self.field)(&rm_item), &pos);

        let curr_item = &self.items[pos];
        self.store.delete((self.field)(curr_item), &last_idx);
        self.store.insert((self.field)(curr_item), pos);

        Some(rm_item)
    }

    pub fn idx(&self) -> Retriever<'_, S, Vec<I>> {
        Retriever::new(&self.store, &self.items)
    }
}

impl<S, I, F> Deref for IList<S, I, F> {
    type Target = [I];

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::IntIndex;
    use rstest::{fixture, rstest};

    #[derive(PartialEq, Debug)]
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

    #[test]
    fn check() {
        let mut l = IList::<IntIndex, Person, _>::new(|p| p.id);
        l.push(Person::new(0, "Paul"));
        l.push(Person::new(-2, "Mario"));
        l.push(Person::new(2, "Jasmin"));

        // retrieve GET
        {
            let mut it = l.idx().get(&-2);
            assert_eq!(Some(&Person::new(-2, "Mario")), it.next());
            assert_eq!(None, it.next());
        }
        // deref
        assert_eq!(3, l.len());
        assert_eq!(Some(&Person::new(-2, "Mario")), l.get(1));
        assert_eq!(&Person::new(-2, "Mario"), &l[1]);

        // update name
        assert_eq!(&Person::new(0, "Paul"), &l[0]); // before
        assert_eq!(
            Some(&Person::new(0, "Egon")),
            l.update(0, |p| p.name = "Egon".into())
        );
        assert_eq!(&Person::new(0, "Egon"), &l[0]); // after

        // update id
        assert_eq!(Some(&Person::new(99, "Egon")), l.update(0, |p| p.id = 99));
        assert_eq!(&Person::new(99, "Egon"), &l[0]); // after
        assert_eq!(&Person::new(99, "Egon"), l.idx().get(&99).next().unwrap());

        // update id and name
        assert_eq!(
            Some(&Person::new(100, "Inge")),
            l.update(0, |p| {
                p.id = 100;
                p.name = "Inge".into()
            })
        );
        assert_eq!(&Person::new(100, "Inge"), l.idx().get(&100).next().unwrap());

        // update invalid
        assert_eq!(None, l.update(10_000, |p| p.id = 99));
    }

    #[fixture]
    fn persons() -> Vec<Person> {
        vec![
            Person::new(0, "Paul"),
            Person::new(-2, "Mario"),
            Person::new(2, "Jasmin"),
        ]
    }

    #[rstest]
    fn remove_0(persons: Vec<Person>) {
        let mut l = IList::<IntIndex, Person, _>::from_iter(|p| p.id, persons.into_iter());
        assert_eq!(&Person::new(0, "Paul"), &l[0]);
        assert_eq!(3, l.len());

        assert_eq!(Some(Person::new(0, "Paul")), l.remove(0));

        assert_eq!(&Person::new(2, "Jasmin"), &l[0]);
        assert_eq!(2, l.len());
        assert_eq!(None, l.idx().get(&0).next());
    }

    #[rstest]
    fn remove_1(persons: Vec<Person>) {
        let mut l = IList::<IntIndex, Person, _>::from_iter(|p| p.id, persons.into_iter());
        assert_eq!(&Person::new(-2, "Mario"), &l[1]);
        assert_eq!(3, l.len());

        assert_eq!(Some(Person::new(-2, "Mario")), l.remove(1));

        assert_eq!(&Person::new(2, "Jasmin"), &l[1]);
        assert_eq!(2, l.len());
        assert_eq!(None, l.idx().get(&-2).next());
    }

    #[rstest]
    fn remove_last_2(persons: Vec<Person>) {
        let mut l = IList::<IntIndex, Person, _>::from_iter(|p| p.id, persons.into_iter());
        assert_eq!(&Person::new(2, "Jasmin"), &l[2]);
        assert_eq!(3, l.len());

        assert_eq!(Some(Person::new(2, "Jasmin")), l.remove(2));

        assert_eq!(2, l.len());
        assert_eq!(None, l.idx().get(&2).next());
    }

    #[rstest]
    fn remove_invalid(persons: Vec<Person>) {
        let mut l = IList::<IntIndex, Person, _>::from_iter(|p| p.id, persons.into_iter());
        assert_eq!(None, l.remove(10_000));

        assert_eq!(3, l.len());
    }

    #[test]
    fn remove_empty() {
        let mut l = IList::<IntIndex, Person, _>::from_iter(|p| p.id, vec![].into_iter());
        assert_eq!(None, l.remove(0));
    }
}
