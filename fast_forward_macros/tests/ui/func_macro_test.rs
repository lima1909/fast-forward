use fast_forward_macros::create_indexed_list;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Car(usize, String);

create_indexed_list!(
    create Cars on Car using {
        id: fast_forward::index::uint::UIntIndex => 0,
        name: fast_forward::index::map::MapIndex => 1,
    }
);

fn main() {
    let v = vec![Car(1, "BMW".into()), Car(2, "VW".into())];
    let cars = Cars::borrowed(&v);

    assert!(cars.id.idx().contains(&2));
    assert!(cars.name.idx().contains(&"BMW".into()));
}
