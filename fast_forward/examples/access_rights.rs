#![allow(dead_code, unused_variables)]

#[derive(Debug, Clone, PartialEq)]
pub struct Car {
    id: usize,
    name: &'static str,
}

fn main() {
    // let l: ROIndexList<'_, _, UIntIndex> = ROIndexList::owned(
    //     |c| c.id,
    //     vec![
    //         Car {
    //             id: 99,
    //             name: "BMW 1",
    //         },
    //         Car {
    //             id: 2043,
    //             name: "Audi",
    //         },
    //         Car {
    //             id: 99,
    //             name: "BMW 2",
    //         },
    //     ],
    // );

    // let idx = l.idx();
    // let mut it = idx.create_view([1, 3, 99]).filter(|c| c.id < 10_000);

    // assert_eq!(
    //     Some(&Car {
    //         id: 99,
    //         name: "BMW 1",
    //     }),
    //     it.next()
    // );
    // assert_eq!(
    //     Some(&Car {
    //         id: 99,
    //         name: "BMW 2",
    //     }),
    //     it.next()
    // );
    // assert_eq!(None, it.next());

    // TODO: it does not work!
    // 2043 is NOT found, it is filter out
    // assert!(idx.create_view(1..=99).contains(&2043))
}
