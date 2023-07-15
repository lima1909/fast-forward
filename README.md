# Fast-Forward [![Build Status]][Build Action] [![Coverage Status]][Coverage Action]

[Build Status]: https://github.com/lima1909/fast-forward/actions/workflows/continuous_integration.yml/badge.svg
[Build Action]: https://github.com/lima1909/fast-forward/actions
[Coverage Status]: https://codecov.io/gh/lima1909/fast-forward/branch/main/graph/badge.svg?token=VO3VV8BFLN
[Coverage Action]: https://codecov.io/gh/lima1909/fast-forward


⏩ Quering lists blazing fast.

This is a very, very, ... early state. This means, this implementation is on the way to find out, what is a good solution 
and want anyone use it. The API can change a lot! Please, try it out and give me feedback.

# Overview

__Fast-Forward__ is a library for finding or filtering items in a (large) collection (Vec, Map, ...), __faster__  than an `Iterator` or a search algorithm.
It is not a replacement of the `Iterator` or searching, is more of an addition.

This faster is achieved  by using `Indices`. This means, it does not have to touch and compare every item in the collection.

An `Index` has two parts, a `Key` (item to searching for) and a `Position` (the index) in the collection.

### Example for an indexed read only List (ro::IList):

```rust
use fast_forward::{index::uint::UIntIndex, collections::ro::IList};

#[derive(Debug, PartialEq)]
pub struct Car(usize, String);

// created an indexed List with the UIntIndex on the Car property 0.
let l = IList::<UIntIndex, _>::new(|c: &Car| c.0, vec![
                            Car(1, "BMW".into()),
                            Car(2, "VW".into())]);

// idx method pointed to the Car.0 property Index and
// gives access to the `Retriever` object to handle queries, like: contains, get, filter.
assert!(l.idx().contains(&2));
assert!(!l.idx().contains(&2000));

// get a Car with the ID = 2
assert_eq!(
    l.idx().get(&2).collect::<Vec<_>>(),
    vec![&Car(2, "VW".into())],
);

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

All supported options for retrieve Items can you find by the [`crate::collections::Retriever`] struct.

### Example for a `View` of an indexed read only List (ro::IList):

A `View` is like a database view. This means you get a subset of items, which you can see.
It is useful, if you don't want to give full read access to the complete collection.

```rust
use fast_forward::{index::uint::UIntIndex, collections::ro::IList};

#[derive(Debug, PartialEq)]
pub struct Car(usize, String);

// created an indexed List with the UIntIndex on the Car property 0.
let l = IList::<UIntIndex, _>::new(|c: &Car| c.0, vec![
                            Car(1, "BMW".into()),
                            Car(2, "VW".into()),
                            Car(3, "Audi".into())]);

// create a view: only for Car ID = 1 0r 3
let view = l.idx().create_view([1, 3]);

// Car with ID 2 is not in the view
assert!(!view.contains(&2));

// the original list contains of course the Car with ID 2
assert!(l.idx().contains(&2));
```

This library consists of the following parts (modules):
- [`crate::index`]: to store Indices and the Indices themself
- [`crate::collections`]: the implementations of indexed collections (e.g. read only: IList, IRefList, IMap).


<hr>
Current version: 0.0.1

License: MIT
