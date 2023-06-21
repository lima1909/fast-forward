use fast_forward_macros::create_indexed_list;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Car(usize, String);

create_indexed_list!(
    create CarsBorrow on Car using {
        id: fast_forward::index::uint::UIntIndex => 0,
        name: fast_forward::index::map::MapIndex => 1.clone,
    }
);

create_indexed_list!(
    create CarsOwned on Car using {
        id: fast_forward::index::uint::UIntIndex => 0,
    }
);

fn main() {
    let v = vec![Car(1, "BMW".into()), Car(2, "VW".into())];

    // Borrowed
    let cars = CarsBorrow::borrowed(&v);

    assert!(cars.id.idx().contains(&2));
    assert!(cars.name.idx().contains(&"BMW".into()));

    // Owned
    let cars = CarsOwned::owned(v);
    assert!(cars.id.idx().contains(&2));
}
