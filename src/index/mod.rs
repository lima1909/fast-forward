//! An Index has the function to find a specific item in a list (Slice, Vec, ...) faster.
//! This means, it does not have to touch and compare every item in the list.
//!
//! An Index has two parts, a `Key` (item to search for) and a `Position` (the index in the list).
//!
//! There are two types of Index:
//! - `Unique Index`: for a `Key` exist exactly one `Position`
//! - `Multi Index`: for a `Key` exists many `Position`s
//!
//! # Example for an Vec-Mulit-Index:
//!
//! Map-Index:
//!
//! - `Key`      = name (String)
//! - `Position` = index in Vec
//!
//! ```java
//! let _names = vec!["Paul", "Jasmin", "Inge", "Paul", ...];
//!
//!  Key (name)   | Position (index in Vec)
//! ----------------------------------------
//!  "Jasmin"     |      1
//!  "Paul"       |      0, 3
//!  "Inge"       |      2
//!   ...         |     ...
//! ```

#![allow(dead_code)]
pub mod error;
pub mod uint;

pub use error::IndexError;
use std::marker::PhantomData;

type Result<T = ()> = std::result::Result<T, IndexError>;

/// A wrapper for supported Index-Types.
#[derive(Debug, Clone, PartialEq)]
pub enum Key {
    Number(Number),
    String(String),
}
impl From<usize> for Key {
    fn from(value: usize) -> Self {
        Key::Number(Number::Usize(value))
    }
}

impl From<String> for Key {
    fn from(value: String) -> Self {
        Key::String(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
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

/// Pos is the index in a List ([`std::vec::Vec`])
pub type Pos = usize;

/// 0, 1 or many [`Pos`]
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Positions(Vec<Pos>);

impl Positions {
    fn from_vec(indices: Vec<Pos>) -> Self {
        Self(indices)
    }

    fn is_none(&self) -> bool {
        self.0.is_empty()
    }

    fn add(&mut self, pos: Pos) {
        self.0.push(pos);
    }

    fn pos(&self) -> &[Pos] {
        self.0.as_slice()
    }

    fn unique(&self) -> Option<&Pos> {
        self.0.get(0)
    }
}

/// A Store for Indices. It's a mapping from a given [`Index`] to a position in a List.
pub trait Store {
    fn insert(&mut self, k: &Key, p: Pos) -> Result;
    fn filter(&self, k: &Key, op: &str) -> Result<&Positions>;
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
pub struct Indices<T, F>(Vec<NamedStore<T, F>>);

impl<T, F> Indices<T, F> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add(&mut self, name: &'static str, store: Box<dyn Store>, get_field_value: F) {
        self.0.push(NamedStore::new(name, store, get_field_value));
    }

    fn insert_index<I>(&mut self, idx_name: &str, t: &T, pos: Pos) -> Result
    where
        I: Into<Key>,
        F: Fn(&T) -> I,
    {
        for s in &mut self.0 {
            if s.name == idx_name {
                let idx = (s.get_field_value)(t);
                s.store.insert(&idx.into(), pos)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{uint::UIntIndexStore, *};

    struct Person(usize, &'static str);

    #[test]
    fn person_indices() {
        let mut indices = Indices::new();
        indices.add(
            "pk",
            Box::new(UIntIndexStore::new_unique()),
            |p: &Person| p.0,
        );
        indices.insert_index("pk", &Person(3, "Jasmin"), 0).unwrap();
    }
}
