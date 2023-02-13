//! The purpose of an Index is to find faster a specific item in a list (Slice, Vec, ...).
//! This means, it does not have to touch and compare every item in the list.
//!
//! An Index has two parts, a [`Key`] (item to search for) and a position (the index in the list) [`Idx`].
//!
//! There are two types of Index:
//! - [`UniqueIdx`]: for a given [`Key`] exist exactly one [`Idx`]
//! - [`AmbiguousIdx`]: for a given [`Key`] exists many [`Idx`]s
//!
//! # Example for an Vec-Ambiguous-Index:
//!
//! Map-Index:
//!
//! - [`Key`] = name (String)
//! - [`Idx`] = index in Vec
//!
//! ```java
//! let _names = vec!["Paul", "Jasmin", "Inge", "Paul", ...];
//!
//!  Key       | Idx
//! -------------------
//!  "Jasmin"  | 1
//!  "Paul"    | 0, 3
//!  "Inge"    | 2
//!   ...      | ...
//! ```

#![allow(dead_code)]
pub mod error;
pub mod uint;

pub use error::IndexError;
use std::{marker::PhantomData, ops::Index};

type Result<T = ()> = std::result::Result<T, IndexError>;

/// Is the value and type for searching an item.
#[derive(Debug, Clone, PartialEq)]
pub enum Key {
    Number(Number),
    String(String),
}

/// [`Key`] of type [`Number`].
#[derive(Debug, Clone, PartialEq)]
pub enum Number {
    Usize(usize),
    I32(i32),
    F32(f32),
}

/// Idx is the index/position in a List ([`std::vec::Vec`]).
pub type Idx = usize;

/// Unique index, has one [`Idx`].
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct UniqueIdx([Idx; 1]);

impl From<Idx> for UniqueIdx {
    fn from(i: Idx) -> Self {
        Self([i])
    }
}

/// Ambiguous indices, has a list of [`Idx`]s.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct AmbiguousIdx(Vec<Idx>);

impl AmbiguousIdx {
    fn new(i: Idx) -> Self {
        Self(vec![i])
    }

    fn push(&mut self, i: Idx) {
        self.0.push(i);
    }
}

impl From<Vec<Idx>> for AmbiguousIdx {
    fn from(v: Vec<Idx>) -> Self {
        Self(v)
    }
}

/// Common interface for getting the [`Idx`]s from [`UniqueIdx`] and [`AmbiguousIdx`].
pub trait AsIdxSlice {
    fn as_idx_slice(&self) -> &[Idx];
}

impl AsIdxSlice for UniqueIdx {
    fn as_idx_slice(&self) -> &[Idx] {
        &self.0
    }
}

impl AsIdxSlice for AmbiguousIdx {
    fn as_idx_slice(&self) -> &[Idx] {
        &self.0
    }
}

/// A Store for Indices. It's a mapping from a given [`Index`] to a position in a List.
pub trait Store: Index<(Key, &'static str), Output = [Idx]> {
    fn insert(&mut self, k: &Key, i: Idx) -> Result;
}

struct NamedStore<T, F> {
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

/// Collection of indices ([`Store`]s).
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

impl Key {
    fn get_usize(&self) -> Result<usize> {
        match self {
            Key::Number(n) => n.get_usize(),
            Key::String(_) => Err(IndexError::InvalidKeyType {
                expected: "usize",
                got: "String",
            }),
        }
    }
}

impl Number {
    fn get_usize(&self) -> Result<usize> {
        match self {
            Number::Usize(u) => Ok(*u),
            Number::I32(i) => TryFrom::try_from(*i).map_err(|_| IndexError::InvalidKeyType {
                expected: "usize",
                got: "i32",
            }),
            Number::F32(_) => Err(IndexError::InvalidKeyType {
                expected: "usize",
                got: "f32",
            }),
        }
    }
}

impl From<String> for Key {
    fn from(value: String) -> Self {
        Key::String(value)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        uint::{U32Index, Unique},
        *,
    };

    struct Person(usize, &'static str);

    #[test]
    fn person_indices() {
        let mut indices = Indices::new();
        indices.add("pk", Box::<U32Index<Unique>>::default(), |p: &Person| p.0);
        indices.insert_index("pk", &Person(3, "Jasmin"), 0).unwrap();
    }
}
