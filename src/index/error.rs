//! Possible `errors` by using index.
use std::{
    error::Error,
    fmt::{Debug, Display},
};

use super::Key;

// use super::Pos;

#[derive(Debug, Clone, PartialEq)]
pub enum IndexError {
    OutOfBound(Key),
    NotUnique(Key),
}

impl Display for IndexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IndexError::OutOfBound(k) => write!(f, "Index: {k:?} out of bound"),
            IndexError::NotUnique(k) => write!(f, "Index: {k:?} is not unique"),
        }
    }
}

impl Error for IndexError {}
