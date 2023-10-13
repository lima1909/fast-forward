# Fast-Forward [![Build Status]][Build Action] [![Coverage Status]][Coverage Action] [![Latest Version]][crates.io]  

[Build Status]: https://github.com/lima1909/fast-forward/actions/workflows/continuous_integration.yml/badge.svg
[Build Action]: https://github.com/lima1909/fast-forward/actions
[Coverage Status]: https://codecov.io/gh/lima1909/fast-forward/branch/main/graph/badge.svg?token=VO3VV8BFLN
[Coverage Action]: https://codecov.io/gh/lima1909/fast-forward
[Latest Version]: https://img.shields.io/crates/v/fast_forward.svg
[crates.io]: https://crates.io/crates/fast_forward


⏩ Quering lists blazing fast.

This is a very, very, ... early state. This means, this implementation is on the way to find out, what is a good solution 
and want anyone use it. The API can change a lot! Please, try it out and give me feedback.

# Overview

__Fast-Forward__ is a library for finding or filtering items in a (large) collection (Vec, Slice, Map, ...).
This means faster than an `iterator` or a `search algorithm`.
It is a wrapper, which extends the given collection with very fast find operations.
This wrapper is just as easy to use as the given (original) collection.

This faster is achieved  by using `Indices`. This means, it does not have to touch and compare every item in the collection.

An `Index` has two parts, a `Key` (item to searching for) and a `Position` (the index) in the collection.

### Example for an indexed read only List (ro::IList):

```rust
use fast_forward::{index::UIntIndex, collections::ro::IList};

#[derive(Debug, PartialEq)]
pub struct Car(usize, String);

// created an indexed List with the UIntIndex on the Car property 0.
let l = IList::<UIntIndex, _>::new(|c: &Car| c.0, vec![
                            Car(1, "BMW".into()),
                            Car(2, "VW".into())]);

// idx method pointed to the Car.0 property Index and
// gives access to the `Retriever` object to handle queries,
// like: contains, get, filter.
assert!(l.idx().contains(&2));
assert!(!l.idx().contains(&2000));

// get a Car with the ID = 2
assert_eq!(l.idx().get(&2).next(), Some(&Car(2, "VW".into())));

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

// you can use the Vec methods too
assert_eq!(2, l.len());

// or you can get MetaData like min and max Key value
use fast_forward::index::store::MetaData;

assert_eq!(1, l.idx().meta().min_key());
assert_eq!(2, l.idx().meta().max_key());
```

All supported options for retrieve Items can you find by the [`crate::collections::Retriever`] struct.

### Example for a `View` of an indexed read only List (ro::IList):

A `View` is like a database view. This means you get a subset of items, which you can see.
It is useful, if you don't want to give full read access to the complete collection.

All details to [`crate::collections::Retriever::create_view()`]

```rust
use fast_forward::{index::MapIndex, collections::ro::IList};

#[derive(Debug, PartialEq)]
pub struct Car(usize, String);

// created an indexed List with the MapIndex on the Car property 1.
let l = IList::<MapIndex, _>::new(|c: &Car| c.1.clone(), vec![
                            Car(1, "BMW".into()),
                            Car(2, "VW".into()),
                            Car(3, "Audi".into())]);

// create a view: only for Car Name = "BMW" 0r "Audi"
let view = l.idx().create_view(
      [String::from("BMW"), String::from("Audi")]);

// Car with Name "VW" is NOT in the view
assert!(!view.contains(&String::from("VW")));

// get the Care with the name "Audi"
assert_eq!(
    view.get(&String::from("Audi")).collect::<Vec<_>>(),
    vec![&Car(3, "Audi".into())],
);

// the original list contains of course the Car with ID "VW"
assert!(l.idx().contains(&String::from("VW")));
```

