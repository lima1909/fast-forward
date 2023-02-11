#![allow(dead_code)]
pub mod uint;

use std::marker::PhantomData;

/// A wrapper for supported Index-Types.
pub enum Index {
    Number(Number),
    String(String),
}
impl From<usize> for Index {
    fn from(value: usize) -> Self {
        Index::Number(Number::Usize(value))
    }
}

impl From<String> for Index {
    fn from(value: String) -> Self {
        Index::String(value)
    }
}

pub enum Number {
    Usize(usize),
    I32(i32),
    F32(f32),
}

impl From<usize> for Number {
    fn from(value: usize) -> Self {
        Number::Usize(value)
    }
}

impl From<i32> for Number {
    fn from(value: i32) -> Self {
        Number::I32(value)
    }
}

impl From<f32> for Number {
    fn from(value: f32) -> Self {
        Number::F32(value)
    }
}

/// 0, 1 or many [`Pos`]
pub struct Positions;

/// Pos is the index in a List ([`std::vec::Vec`])
pub type Pos = usize;

/// A Store for Indices. It's a mapping from a given [`Index`] to a position in a List.
pub trait Store {
    fn insert(&mut self, idx: Index, pos: Pos);
    fn filter(&self, val: &Index, op: &str) -> Positions;
}

pub struct NamedStore<T, F> {
    name: &'static str,
    store: Box<dyn Store>,
    get_field_value: F,
    _type: PhantomData<T>,
}

impl<T, F> NamedStore<T, F> {
    pub fn new(name: &'static str, store: Box<dyn Store>, get_field_value: F) -> Self {
        Self {
            name,
            store,
            get_field_value,
            _type: PhantomData,
        }
    }
}

#[derive(Default)]
pub struct Indices<T, F> {
    indices: Vec<NamedStore<T, F>>,
}

impl<T, F> Indices<T, F> {
    pub fn new() -> Self {
        Self {
            indices: Vec::new(),
        }
    }

    pub fn add(&mut self, name: &'static str, store: Box<dyn Store>, get_field_value: F) {
        self.indices
            .push(NamedStore::new(name, store, get_field_value));
    }

    fn insert_index<I>(&mut self, idx_name: &str, t: &T, pos: Pos)
    where
        I: Into<Index>,
        F: Fn(&T) -> I,
    {
        for s in &mut self.indices {
            if s.name == idx_name {
                let idx = (s.get_field_value)(t);
                s.store.insert(idx.into(), pos);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn it_works() {
        // struct Person(usize);

        // let mut indices = Indices::new();
        // indices.add("pk", Box::new(ListIndex), |p: &Person| p.0);
        // indices.insert_index("pk", &Person(3), 0);
    }
}
