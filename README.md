# fast_forward

__Fast-Forward__ is a library for finding or filtering items in a (large) collection (Vec, Map, ...), __faster__  than an `Iterator` or a search algorithm.
It is not a replacement of the `Iterator` or searching, is more of an addition.

This faster is achieved  by using `Indices`. This means, it does not have to touch and compare every item in the collection.

An `Index` has two parts, a `Key` (item to searching for) and a `Position` (the index) in the collection.

### Example for an indexed read only List (ro::IList):

```rust
use fast_forward::{index::uint::UIntIndex, collections::ro::IList};

#[derive(Debug, PartialEq)]
pub struct Car(usize, String);

// create a list of Cars
let cars = vec![Car(1, "BMW".into()), Car(2, "VW".into())];

// created an indexed List with the UIntIndex on the Car property 0.
let l = IList::<UIntIndex, _>::new(|c: &Car| c.0, cars);

// idx method pointed to the Car.0 property Index and
// gives access to the `Retriever` object to handle queries, like: contains, get, filter.
assert!(l.idx().contains(&2));
assert!(!l.idx().contains(&2000));

// get a Car with the ID = 2
let mut it = l.idx().get(&2);
assert_eq!(Some(&Car(2, "VW".into())), it.next());

// get many Cars with ID = 2 or 1
assert_eq!(
    l.idx().get_many([2, 1]).collect::<Vec<_>>(),
    vec![&Car(2, "VW".into()), &Car(1, "BMW".into())],
);

// the same query with the filter-method
// (which has the disadvantage, that this need a allocation)
assert_eq!(
    l.idx().filter(|f| f.eq(&2) | f.eq(&1)).collect::<Vec<_>>(),
    vec![&Car(1, "BMW".into()), &Car(2, "VW".into())],
);
```

All supported options for retrieve Items can you find by [`crate::collections::Retriever`].

Tis library consists of the following parts (modules):
- [`crate::index`]: to store Indices and the Indices themself
- [`crate::collections`]: the implementations of indexed collections (e.g. read only: IList, IRefList, IMap).


License: MIT
