//! An Index has the function to find a specific item in a list (Slice, Vec, ...) faster.
//! This means, it does not have to touch and compare every item in the list.
//!
//! An Index has two parts, a `Key` (item to search for) and a `Position` (the index in the list).
//!
//! There are two types of Index:
//! - `Unique Index`: for a `Key` exist exactly one `Position`
//! - `Ambiguous Index`: for a `Key` exists many `Position`s
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
use std::{marker::PhantomData, ops::Index};

type Result<T = ()> = std::result::Result<T, IndexError>;

/// A wrapper for supported Index-Types.
#[derive(Debug, Clone, PartialEq)]
pub enum Key {
    Number(Number),
    String(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Number {
    Usize(usize),
    I32(i32),
    F32(f32),
}

/// Idx is the index in a List ([`std::vec::Vec`])
pub type Idx = usize;

pub trait AsSlice {
    fn as_slice(&self) -> &[Idx];
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct UniqueIndex([Idx; 1]);

impl From<Idx> for UniqueIndex {
    fn from(i: Idx) -> Self {
        Self([i])
    }
}

impl AsSlice for UniqueIndex {
    fn as_slice(&self) -> &[Idx] {
        &self.0
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct AmbiguousIndex(Vec<Idx>);

impl AmbiguousIndex {
    fn new(i: Idx) -> Self {
        Self(vec![i])
    }

    fn push(&mut self, i: Idx) {
        self.0.push(i);
    }
}

impl From<Vec<Idx>> for AmbiguousIndex {
    fn from(v: Vec<Idx>) -> Self {
        Self(v)
    }
}

impl AsSlice for AmbiguousIndex {
    fn as_slice(&self) -> &[Idx] {
        &self.0
    }
}

/// A Store for Indices. It's a mapping from a given [`Index`] to a position in a List.
pub trait Store: Index<(Key, &'static str), Output = [Idx]> {
    fn insert(&mut self, k: &Key, i: Idx) -> Result;
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

    fn insert_index<I>(&mut self, idx_name: &str, t: &T, idx: Idx) -> Result
    where
        I: Into<Key>,
        F: Fn(&T) -> I,
    {
        for s in &mut self.0 {
            if s.name == idx_name {
                let key = (s.get_field_value)(t);
                s.store.insert(&key.into(), idx)?;
            }
        }

        Ok(())
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! into_key {
    ( $as:ty : $($t:ty), + => $key_t:tt ) => {
        $(
        impl From<$t> for $crate::index::Number {
            fn from(val: $t) -> Self {
                $crate::index::Number::$key_t(val as $as)
            }
        }

        impl From<$t> for $crate::index::Key {
            fn from(val: $t) -> Self {
                $crate::index::Key::Number($crate::index::Number::$key_t(val as $as))
            }
        }

        )+
    };

}

into_key!(usize : usize, u8, u32, u64  => Usize);
into_key!(i32   : i8, i32, i64 => I32);
into_key!(f32   : f32, f64 => F32);

impl From<String> for Key {
    fn from(value: String) -> Self {
        Key::String(value)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        uint::{UIntIndexStore, UniqueListIndex},
        *,
    };

    struct Person(usize, &'static str);

    #[test]
    fn person_indices() {
        let mut indices = Indices::new();
        indices.add(
            "pk",
            Box::new(UIntIndexStore::<UniqueListIndex>::default()),
            |p: &Person| p.0,
        );
        indices.insert_index("pk", &Person(3, "Jasmin"), 0).unwrap();
    }
}
