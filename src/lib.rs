//! **Fast-Forward** is a library for filtering items in a (large) list, _faster_ than an `Iterator` ([`std::iter::Iterator::filter`]).
//!
//! This _faster_ is achieved  by using `Indices`. This means, it does not have to touch and compare every item in the list.
//!
//! An Index has two parts, a `Key` (item to searching for) and a position (the index in the list) [`Idx`].
//!
//! ## A simple Example:
//!
//! ```text
//! let _list_with_names = vec!["Paul", "Jon", "Inge", "Paul", ...];
//! ```
//!
//! Index `Map(name, idx's)`:
//!
//! ```text
//!  Key     | Idx
//! ---------------
//!  "Paul"  | 0, 3
//!  "Jon"   | 1
//!  "Inge"  | 2
//!   ...    | ...
//! ```
//!
//! To Find the `Key`: "Jon" with the `operation = equals` is only one step necessary.
//!
pub mod error;
pub mod index;
pub mod query;

pub use query::query;

/// Default Result for index with the Ok(T) value or en [`error::Error`].
pub type Result<T = ()> = std::result::Result<T, error::Error>;

/// `Idx` is the index/position in a List ([`std::vec::Vec`]).
pub type Idx = usize;

#[macro_export]
macro_rules! fast {
    (   $strukt:ident$(<$lt:lifetime>)?
        {
            $( $fast_field:ident $(.$func:ident)?: $typ:ty ), + $(,)*
        }
    ) => {
        fast!($strukt$(<$lt>)? as Fast { $( $fast_field $(.$func)?: $typ ), + })
    };

    (   $strukt:ident$(<$lt:lifetime>)? as $fast:ident
        {
            $( $fast_field:ident $(.$func:ident)?: $typ:ty ), + $(,)*
        }

    ) => {

        {

        /// Container-struct for all indices.
        #[derive(Default)]
        struct $fast$(<$lt>)? {
            $(
                $fast_field: $typ,
            )+
        }

        /// Insert in all indices-stores the `Key` and the `Index`.
        impl$(<$lt>)? $fast$(<$lt>)? {
            fn insert(&mut self, s: &$($lt)? $strukt, idx: $crate::Idx) -> $crate::Result {
                use $crate::index::Store;

                $(
                    self.$fast_field.insert(s.$fast_field$(.$func())?, idx)?;
                )+


                Ok(())
            }
        }

        $fast::default()

        }

    };
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Person {
        id: usize,
        _multi: usize,
        name: String,
    }

    #[test]
    fn fast() {
        use crate::index::{map::UniqueStrIdx, uint::PkUintIdx};
        use crate::query;

        let mut p = fast!(
                Person<'p> {
                    id: PkUintIdx,
                    name.as_ref: UniqueStrIdx<'p>,
                }
        );

        let p1 = Person {
            id: 4,
            _multi: 8,
            name: "Foo".into(),
        };
        p.insert(&p1, 1).unwrap();

        assert_eq!([1], *query(p.id.eq(4)).or(p.name.eq("Foo")).exec());
    }
}
