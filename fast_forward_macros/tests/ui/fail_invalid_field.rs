use fast_forward_macros::fast;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Car(usize, String);

fast!(
    create Cars on Car using {
        id: fast_forward::index::uint::UIntIndex => id,
    }
);

fn main() {
    let cars = Cars::new(vec![Car(1, "BMW".into())]);
    assert_eq!(Some(&Car(1, "BMW".into())), cars.get(0));
}
