pub mod index;
pub mod ops;
pub mod query;

/// `Idx` is the index/position in a List ([`std::vec::Vec`]).
pub type Idx = usize;

/// Id for operations.
/// Operations are primarily compare functions, like equal, greater than and so on.
pub type Op = u8;
