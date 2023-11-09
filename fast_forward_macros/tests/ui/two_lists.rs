use fast_forward_macros::fast;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Car(usize, String);

fast!(
    create ref_list CarsRef on Car using {
        id: fast_forward::index::MultiUIntIndex => 0,
        name: fast_forward::index::map::MapIndex => 1.clone,
    }
);

fast!(
    create Cars on Car using {
        id: fast_forward::index::MultiUIntIndex => 0,
        name: fast_forward::index::map::MapIndex => 1.clone,
    }
);

// fast!(
//     create ro map CarsMap on Car using {
//         id: fast_forward::index::uint::UIntIndex => 0,
//     }
// );

fn main() {
    let v = vec![Car(1, "BMW".into()), Car(2, "VW".into())];

    // Borrowed
    let cars = CarsRef::new(&v);

    assert!(cars.id().contains(&2));
    assert!(cars.name().contains(&"BMW".into()));
    // deref
    assert_eq!(2, cars.len());
    assert!(cars.contains(&Car(2, "VW".into())));
    assert_eq!(&Car(2, "VW".into()), &cars[1]);

    // ----------------------------
    // Owned
    let cars = Cars::new(v);
    assert!(cars.id().contains(&2));
    assert!(cars.name().contains(&"BMW".into()));
    // deref
    assert_eq!(2, cars.len());
    assert!(cars.contains(&Car(2, "VW".into())));
    assert_eq!(&Car(2, "VW".into()), &cars[1]);

    // ----------------------------
    // combine two indices: id and name
    let idxs = cars.id().eq(&2) & cars.name().eq(&"VW".into());
    let mut it = idxs.as_slice().iter().map(|i| &cars[*i]);
    assert_eq!(Some(&Car(2, "VW".into())), it.next());
    assert_eq!(None, it.next());

    // ----------------------------
    // Map
    // let mut m = HashMap::<usize, Car>::new();
    // m.insert(1, Car(1, "BMW".into()));
    // m.insert(2, Car(2, "VW".into()));

    // let cars = CarsMap::new(m);
}
