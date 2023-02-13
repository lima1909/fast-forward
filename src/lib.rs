pub mod index;

/// Id for operations. The default operations are [`DefaultOp`]
pub type Op = u8;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum DefaultOp {
    /// equal =
    Eq = 1,
    /// not equal !=
    Neq = 2,
    /// less equal <=
    Le = 3,
    /// less than <
    Lt = 4,
    /// greater equal >=
    Ge = 5,
    /// greater than >
    Gt = 6,
}

impl DefaultOp {
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}
