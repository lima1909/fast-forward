use fast_forward_macros::create_indexed_list;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Car(usize, String);

create_indexed_list!(
    create Cars on Car using {
        id: UIntIndex => 0,
    }
);

fn main() {
    let cars = Cars::owned(vec![Car(1, "BMW".into())]);
    assert_eq!(Some(&Car(1, "BMW".into())), cars.get(0));
}
