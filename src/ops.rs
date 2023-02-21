//! Operations are primarily compare functions, like equal, greater than and so on.
use crate::{
    query::{Filter, Key},
    Op,
};

/// equal `=`
pub const EQ: Op = 1;
/// not equal `!=`
pub const NE: Op = 2;
/// less than `<`
pub const LT: Op = 3;
/// less equal `<=`
pub const LE: Op = 4;
/// greater than `>`
pub const GT: Op = 5;
/// greater equal `>=`
pub const GE: Op = 6;

/// Equals `Key`
pub fn eq<'a, K: Into<Key<'a>>>(field: &'a str, key: K) -> Filter<'a> {
    Filter::new(field, EQ, key.into())
}

/// Not Equals `Key`
pub fn ne<'a, K: Into<Key<'a>>>(field: &'a str, key: K) -> Filter<'a> {
    Filter::new(field, NE, key.into())
}
