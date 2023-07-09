use fast_forward_macros::indexed_list;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Car(usize, String);

indexed_list!(
    create Cars on Car using {
        id: fast_forward::index::uint::UIntIndex => id,
    }
);

fn main() {
    let cars = Cars::new(vec![Car(1, "BMW".into())]);
    assert_eq!(Some(&Car(1, "BMW".into())), cars.get(0));
}
