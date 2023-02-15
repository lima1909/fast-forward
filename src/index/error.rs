//! Possible `errors` by using index.
use std::{
    error::Error,
    fmt::{Debug, Display},
};

use super::Key;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexError {
    /// Occurs, if the Index is unique and the given key is already used.
    NotUniqueKey(Key),
    /// Occurs, if an other type of key is expected, as the given key.
    InvalidKeyType {
        /// expected key type.
        expected: &'static str,
        /// got key type.
        got: &'static str,
    },
}

impl Display for IndexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IndexError::NotUniqueKey(k) => write!(f, "Index: {k:?} is not unique"),
            IndexError::InvalidKeyType { expected, got } => {
                write!(f, "Invalid key type. Expected {expected} got: {got}")
            }
        }
    }
}

impl Error for IndexError {}
