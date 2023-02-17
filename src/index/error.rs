//! Possible `errors` by using index.
use std::{
    error::Error,
    fmt::{Debug, Display},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexError {
    /// Occurs, if the Index is unique and the given key is already used.
    NotUniqueKey,
}

impl Display for IndexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IndexError::NotUniqueKey => write!(f, "Index is not unique"),
        }
    }
}

impl Error for IndexError {}
