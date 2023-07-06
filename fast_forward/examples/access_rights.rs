#![allow(dead_code, unused_variables)]

use fast_forward::{collections::ro, index::map::MapIndex};

#[derive(Debug, PartialEq)]
pub struct Car {
    id: usize,
    name: String,
}

fn main() {
    let l: ro::IList<MapIndex, _> = ro::IList::from_list(
        |c| c.name.clone(),
        [
            Car {
                id: 99,
                name: "BMW".into(),
            },
            Car {
                id: 7,
                name: "Audi".into(),
            },
            Car {
                id: 6,
                name: "BMW".into(),
            },
            Car {
                id: 1,
                name: "Porsche".into(),
            },
        ],
    );

    // let mut it = l.idx().create_view([1, 3, 99]).filter(|c| c.id < 10_000);
    let view = l
        .idx()
        .create_view([String::from("Porsche"), String::from("BMW")]);

    // no ACL, can not see car Ferrari
    assert!(!view.contains(&String::from("Audi")));
    assert_eq!(None, view.get(&String::from("Audi")).next());
    assert!(view.get_many([String::from("Audi")]).next().is_none());

    assert_eq!(
        3,
        view.get_many([String::from("BMW"), String::from("Porsche")])
            .collect::<Vec<_>>()
            .len()
    );

    let find = String::from("BMW");
    let mut it = view.get(&find);
    assert_eq!(
        Some(&Car {
            id: 99,
            name: "BMW".into(),
        }),
        it.next()
    );
    assert_eq!(
        Some(&Car {
            id: 6,
            name: "BMW".into(),
        }),
        it.next()
    );
    assert_eq!(None, it.next());

    // check with many
    let mut it = view.get_many([String::from("Porsche"), String::from("Ferrari")]);
    assert_eq!(
        Some(&Car {
            id: 1,
            name: "Porsche".into(),
        }),
        it.next()
    );
    assert_eq!(None, it.next());
}
