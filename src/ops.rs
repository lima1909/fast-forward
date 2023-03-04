//! Operations are primarily compare functions, like equal, greater than and so on.
use crate::{
    query::{Key, NamedPredicate},
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
pub fn eq<'k, K: Into<Key<'k>>>(field: &'k str, key: K) -> NamedPredicate<'k> {
    NamedPredicate::new(field, EQ, key.into())
}

/// Not Equals `Key`
pub fn ne<'k, K: Into<Key<'k>>>(field: &'k str, key: K) -> NamedPredicate<'k> {
    NamedPredicate::new(field, NE, key.into())
}
