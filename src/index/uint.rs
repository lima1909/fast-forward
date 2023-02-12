use std::ops::Index;

use super::{IndexError, Key, Pos, Positions, Result, Store};

/// Index for
///
/// Well suitable for `unsigned integer (u32)` ( for example Primary Keys).
///
///```java
/// let _primary_keys = vec![1, 2, 3, ...];
///
/// PrimaryKey | Position
/// ----------------------
///     0      |   -
///     1      |   0
///     2      |   1
///     3      |   2
///    ...     |  ...
/// ```
#[derive(Debug, Default)]
pub struct UIntIndexStore {
    unique: bool,
    default: Positions,
    positions: Vec<Positions>,
}

impl UIntIndexStore {
    pub fn new(unique: bool) -> Self {
        Self {
            unique,
            default: Positions::default(),
            positions: Vec::new(),
        }
    }

    pub fn new_unique() -> Self {
        Self::new(true)
    }

    pub fn new_ambiguous() -> Self {
        Self::new(false)
    }
}

impl Index<(Key, &'static str)> for UIntIndexStore {
    type Output = Positions;

    fn index(&self, key: (Key, &'static str)) -> &Self::Output {
        if key.1 != "=" {
            todo!()
        }

        let pos = match key.0 {
            Key::Number(super::Number::Usize(u)) => u,
            Key::Number(super::Number::I32(i)) => usize::try_from(i).ok().unwrap(),
            _ => todo!(),
        };

        if self.positions.len() <= pos {
            return &self.default;
        }

        &self.positions[pos]
    }
}

impl Store for UIntIndexStore {
    fn insert(&mut self, key: &Key, pos: Pos) -> Result {
        let i = match key {
            Key::Number(super::Number::Usize(u)) => *u,
            Key::Number(super::Number::I32(i)) => usize::try_from(*i).ok().unwrap(),
            _ => todo!(),
        };

        if self.positions.len() <= i {
            self.positions.resize(i + 1, Positions::default());
        }

        if self.unique && !self.positions[i].is_none() {
            return Err(IndexError::NotUnique(key.clone()));
        }
        self.positions[i].add(pos);

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    mod unique {
        use super::super::*;

        #[test]
        fn empty() {
            let idx = UIntIndexStore::new_unique();
            assert!(idx.index((2.into(), "=")).is_none());
            assert!(idx.positions.is_empty());
        }

        #[test]
        fn find_idx_2() {
            let mut idx = UIntIndexStore::new_unique();
            idx.insert(&2.into(), 2).unwrap();

            assert_eq!(Positions::from_pos(2), idx[(2.into(), "=")]);
            assert_eq!(3, idx.positions.len());
        }

        #[test]
        fn double_index() {
            let mut idx = UIntIndexStore::new_unique();
            idx.insert(&2.into(), 2).unwrap();

            assert_eq!(
                Err(IndexError::NotUnique(2.into())),
                idx.insert(&2.into(), 2)
            );
        }

        #[test]
        fn out_of_bound() {
            let idx = UIntIndexStore::new_unique();
            assert_eq!(&Positions::default(), idx.index((2.into(), "=")));
        }
    }

    mod ambiguous {
        use super::super::*;

        #[test]
        fn empty() {
            let idx = UIntIndexStore::new_ambiguous();
            assert_eq!(&Positions::default(), idx.index((2.into(), "=")));
            assert!(idx.positions.is_empty());
        }

        #[test]
        fn find_idx_2() {
            let mut idx = UIntIndexStore::new_ambiguous();
            idx.insert(&2.into(), 2).unwrap();

            assert_eq!(&Positions::from_pos(2), idx.index((2.into(), "=")));
            assert_eq!(3, idx.positions.len());
        }

        #[test]
        fn double_index() {
            let mut idx = UIntIndexStore::new_ambiguous();
            idx.insert(&2.into(), 2).unwrap();
            idx.insert(&2.into(), 1).unwrap();

            assert_eq!(&Positions::from_vec(vec![2, 1]), idx.index((2.into(), "=")));
        }
    }
}
