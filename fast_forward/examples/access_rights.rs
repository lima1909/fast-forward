#![allow(dead_code, unused_variables)]

use fast_forward::{collections::ro, index::imap::MapIndex};

#[derive(Debug, PartialEq)]
pub struct Car {
    id: usize,
    name: String,
}

fn main() {
    let l: ro::IList<MapIndex, _, [Car; 4]> = ro::IList::new(
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

    let view = l.idx().create_view(["Porsche".into(), "BMW".into()]);
    // no ACL for "Audi", so you can not see car "Audi"
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

    // let view = l
    //     .idx()
    //     // no ACL, can not see car Ferrari
    //     .create_view(["Porsche".into(), "BMW".into()], |view| {
    //         assert!(!view.contains(&String::from("Audi")));
    //         assert_eq!(None, view.get(&String::from("Audi")).next());
    //         assert!(view.get_many([String::from("Audi")]).next().is_none());

    //         assert_eq!(
    //             3,
    //             view.get_many([String::from("BMW"), String::from("Porsche")])
    //                 .collect::<Vec<_>>()
    //                 .len()
    //         );

    //         let find = String::from("BMW");
    //         let mut it = view.get(&find);
    //         assert_eq!(
    //             Some(&Car {
    //                 id: 99,
    //                 name: "BMW".into(),
    //             }),
    //             it.next()
    //         );
    //         assert_eq!(
    //             Some(&Car {
    //                 id: 6,
    //                 name: "BMW".into(),
    //             }),
    //             it.next()
    //         );
    //         assert_eq!(None, it.next());

    //         // check with many
    //         let mut it = view.get_many([String::from("Porsche"), String::from("Ferrari")]);
    //         assert_eq!(
    //             Some(&Car {
    //                 id: 1,
    //                 name: "Porsche".into(),
    //             }),
    //             it.next()
    //         );
    //         assert_eq!(None, it.next());
    //     });
}
