use super::{Index, Pos, Positions, Store};

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
struct ListIndex;

impl Store for ListIndex {
    fn insert(&mut self, _idx: Index, _pos: Pos) {}

    fn filter(&self, _val: &Index, _op: &str) -> Positions {
        todo!()
    }
}
