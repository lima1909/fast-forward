//! Possible `errors` by using index.
use std::{
    error::Error,
    fmt::{Debug, Display},
};

use super::Key;

#[derive(Debug, Clone, PartialEq)]
pub enum IndexError {
    NotUnique(Key),
}

impl Display for IndexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IndexError::NotUnique(k) => write!(f, "Index: {k:?} is not unique"),
        }
    }
}

impl Error for IndexError {}
