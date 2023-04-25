# Fast-Forward [![Build Status]][Build Action] [![Coverage Status]][Coverage Action]

[Build Status]: https://github.com/lima1909/fast-forward/actions/workflows/continuous_integration.yml/badge.svg
[Build Action]: https://github.com/lima1909/fast-forward/actions
[Coverage Status]: https://codecov.io/gh/lima1909/fast-forward/branch/main/graph/badge.svg?token=VO3VV8BFLN
[Coverage Action]: https://codecov.io/gh/lima1909/fast-forward


⏩ Quering lists blazing fast.

This is a very, very, ... early state. This means, this implementation is on the way to find out, what is a good solution 
and want anyone use it. The API can change a lot! Please, try it out and give me feedback.

# Overview

**Fast-Forward** is a library for filtering items in a (large) list, _faster_ than an `Iterator` ([`std::iter::Iterator::filter`]).
It is not a replacement of the `Iterator`, but an addition.

This _faster_ is achieved  by using `Indices`. This means, it does not have to touch and compare every item in the list.

An `Index` has two parts, a `Key` (item to searching for) and a `Position` (the index) in the list.

### A simple Example:

```
let _list_with_names = vec!["Paul", "Jon", "Inge", "Paul", ...];
```

Index `Map(name, idx's)`:

```
 Key     | Idx
---------------
 "Paul"  | 0, 3
 "Jon"   | 1
 "Inge"  | 2
  ...    | ...
```

To Find the `Key`: "Jon" with the `operation equals` is only one step necessary.


<hr>
Current version: 0.1.0

License: MIT
