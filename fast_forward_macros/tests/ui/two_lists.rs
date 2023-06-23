use fast_forward_macros::indexed_list;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Car(usize, String);

indexed_list!(
    create CarsBorrow on Car using {
        id: fast_forward::index::uint::UIntIndex => 0,
        name: fast_forward::index::map::MapIndex => 1.clone,
    }
);

indexed_list!(
    create CarsOwned on Car using {
        id: fast_forward::index::uint::UIntIndex => 0,
        name: fast_forward::index::map::MapIndex => 1.clone,
    }
);

fn main() {
    let v = vec![Car(1, "BMW".into()), Car(2, "VW".into())];

    // Borrowed
    let cars = CarsBorrow::borrowed(&v);

    assert!(cars.id().contains(&2));
    assert!(cars.name().contains(&"BMW".into()));
    // deref
    assert_eq!(2, cars.len());
    assert!(cars.contains(&Car(2, "VW".into())));

    // ----------------------------
    // Owned
    let cars = CarsOwned::owned(v);
    assert!(cars.id().contains(&2));
    assert!(cars.name().contains(&"BMW".into()));
    // deref
    assert_eq!(2, cars.len());
    assert!(cars.contains(&Car(2, "VW".into())));

    // ----------------------------
    // combine two indices: id and name
    let idxs = cars.id().eq(&2) & cars.name().eq(&"VW".into());
    let mut it = idxs.items(&cars);
    assert_eq!(Some(&Car(2, "VW".into())), it.next());
    assert_eq!(None, it.next());
}
