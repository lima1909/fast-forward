#![allow(dead_code, unused_variables)]

use fast_forward::{collections::ROIndexList, index::uint::UIntIndex};

#[derive(Debug, Clone, PartialEq)]
pub struct Car {
    id: usize,
    name: &'static str,
}

fn main() {
    let l: ROIndexList<'_, _, UIntIndex> = ROIndexList::owned(
        |c| c.id,
        vec![
            Car {
                id: 99,
                name: "BMW 1",
            },
            Car {
                id: 7,
                name: "Audi",
            },
            Car {
                id: 99,
                name: "BMW 2",
            },
            Car {
                id: 1,
                name: "Porsche",
            },
        ],
    );

    // let mut it = l.idx().create_view([1, 3, 99]).filter(|c| c.id < 10_000);
    let view = l.idx().create_view(vec![1, 3, 99].into_iter());

    // no ACL, can not see car 7
    assert!(!view.contains(&7));
    assert!(view.get(&7).is_none());
    assert!(view.get_many([7]).next().is_none());

    assert_eq!(3, view.get_many([1, 99, 7]).collect::<Vec<_>>().len());

    let mut it = view.get(&99).unwrap();
    assert_eq!(
        Some(&Car {
            id: 99,
            name: "BMW 1",
        }),
        it.next()
    );
    assert_eq!(
        Some(&Car {
            id: 99,
            name: "BMW 2",
        }),
        it.next()
    );
    assert_eq!(None, it.next());

    // check with many
    let mut it = view.get_many([99, 7]);
    assert_eq!(
        Some(&Car {
            id: 99,
            name: "BMW 1",
        }),
        it.next()
    );
    assert_eq!(
        Some(&Car {
            id: 99,
            name: "BMW 2",
        }),
        it.next()
    );
    assert_eq!(None, it.next());

    // create new view with Range
    assert!(!l.idx().create_view(10..100).contains(&7))
}
