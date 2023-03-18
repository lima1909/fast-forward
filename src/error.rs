//! Possible `errors` by using index.
use std::fmt::{Debug, Display};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Occurs, if the Index is unique and the given key is already used.
    NotUniqueIndexKey,
    InvalidKeyType {
        expected: &'static str,
        got: String,
    },
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NotUniqueIndexKey => write!(f, "Index-key is not unique"),
            Error::InvalidKeyType { expected, got } => {
                write!(f, "expect key-type: {expected}, got: {got}")
            }
        }
    }
}

impl std::error::Error for Error {}
