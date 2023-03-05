//! **Fast-Forward** is a library for searching items in a (large) list _faster_ than an `Iterator` ([`std::iter::Iterator::filter`]).
//! This _faster_ is achieved  by using `Indices`. This means, it does not have to touch and compare every item in the list.
//!
//! An Index has two parts, a [`Key`] (item to search for) and a position (the index in the list) [`Idx`].
//!
//! ## A simple Example:
//!
//! ```text
//! let _list_with_names = vec!["Paul", "Jasmin", "Inge", "Paul", ...];
//! ```
//!
//! Index `Map(name, idx's)`:
//!
//! ```text
//!  Key       | Idx
//! -------------------
//!  "Paul"    | 0, 3
//!  "Jasmin"  | 1
//!  "Inge"    | 2
//!   ...      | ...
//! ```
//!
//! To Find the [`Key::Str("Jasmin")`] with the [`Op::EQ`] is only one step necessary.

pub mod index;
pub mod query;

/// `Idx` is the index/position in a List ([`std::vec::Vec`]).
pub type Idx = usize;

/// Supported types for quering/filtering [`Predicate`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key<'a> {
    Usize(usize),
    Str(&'a str),
}

impl<'a> From<Key<'a>> for usize {
    fn from(key: Key<'a>) -> Self {
        match key {
            Key::Usize(u) => u,
            _ => todo!(),
        }
    }
}

impl<'a> From<Key<'a>> for &'a str {
    fn from(key: Key<'a>) -> Self {
        match key {
            Key::Str(s) => s,
            _ => todo!(),
        }
    }
}

impl From<usize> for Key<'_> {
    fn from(u: usize) -> Self {
        Key::Usize(u)
    }
}

impl<'a> From<&'a str> for Key<'a> {
    fn from(s: &'a str) -> Self {
        Key::Str(s)
    }
}

/// Operations are primarily compare functions, like equal, greater than and so on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    /// equal `=`
    EQ,
    /// not equal `!=`
    NE,
    /// less than `<`
    LT,
    /// less equal `<=`
    LE,
    /// greater than `>`
    GT,
    /// greater equal `>=`
    GE,
    /// define your own Op
    Other(u8),
}

/// A Predicate is a filter definition.
/// This means, a filter consist of an optional field-name, a operation [`Op`] and a [`Key`] (`name = "Jasmin"`).
/// A [`Key`] is a unique value under which all occurring indices are stored.
///
/// For example:
/// ```text
/// name =  "Jasmin"
/// ```
///  - field-name: `name`
///  - Op: `=`
///  - Key: `"Jasmin"`
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Predicate<'k>(&'k str, Op, Key<'k>);

impl<'k> Predicate<'k> {
    pub const fn new(op: Op, key: Key<'k>) -> Self {
        Self("", op, key)
    }

    pub const fn new_eq(key: Key<'k>) -> Self {
        Self("", Op::EQ, key)
    }
}

/// Shortcut for: `= (usize)`
impl<'k> From<usize> for Predicate<'k> {
    fn from(u: usize) -> Self {
        Predicate::new_eq(Key::Usize(u))
    }
}

/// Shortcut for: `= (&str)`
impl<'k> From<&'k str> for Predicate<'k> {
    fn from(s: &'k str) -> Self {
        Predicate::new_eq(Key::Str(s))
    }
}

/// Shortcut: `field = Key`
pub fn eq<'k, K: Into<Key<'k>>>(field: &'k str, key: K) -> Predicate<'k> {
    Predicate(field, Op::EQ, key.into())
}

/// Shortcut: `field != Key`
pub fn ne<'k, K: Into<Key<'k>>>(field: &'k str, key: K) -> Predicate<'k> {
    Predicate(field, Op::NE, key.into())
}
