# Fast-Forward [![Build Status]][Build Action] [![Coverage Status]][Coverage Action]

[Build Status]: https://github.com/lima1909/fast-forward/actions/workflows/continuous_integration.yml/badge.svg
[Build Action]: https://github.com/lima1909/fast-forward/actions
[Coverage Status]: https://codecov.io/gh/lima1909/fast-forward/branch/main/graph/badge.svg?token=VO3VV8BFLN
[Coverage Action]: https://codecov.io/gh/lima1909/fast-forward


⏩ Quering lists blazing fast.

# Overview

**Fast-Forward** is a library for filtering items in a (large) list, _faster_ than an `Iterator` ([`std::iter::Iterator::filter`]).

This _faster_ is achieved  by using `Indices`. This means, it does not have to touch and compare every item in the list.

An Index has two parts, a [`Key`] (item to searching for) and a position (the index in the list) [`Idx`].

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

To Find the [`Key::Str("Jon")`] with the [`Op::EQ`] is only one step necessary.


<hr>
Current version: 0.1.0

License: MIT
