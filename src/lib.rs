pub mod index;
pub mod query;

pub use index::Predicate;
pub use query::NamedPredicate;

/// Supported types for quering/filtering [`NamedPredicate`] or [`Predicate`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key<'a> {
    Usize(usize),
    Str(&'a str),
}

/// `Idx` is the index/position in a List ([`std::vec::Vec`]).
pub type Idx = usize;

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

/// Equals `Key`
pub fn eq<'k, K: Into<Key<'k>>>(field: &'k str, key: K) -> NamedPredicate<'k> {
    NamedPredicate::new(field, Op::EQ, key.into())
}

/// Not Equals `Key`
pub fn ne<'k, K: Into<Key<'k>>>(field: &'k str, key: K) -> NamedPredicate<'k> {
    NamedPredicate::new(field, Op::NE, key.into())
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
