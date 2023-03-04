use query::{Key, NamedPredicate};

pub mod index;
pub mod query;

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
