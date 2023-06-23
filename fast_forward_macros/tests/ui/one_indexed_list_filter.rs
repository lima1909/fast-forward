use fast_forward_macros::create_indexed_list;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Car(usize, String);

create_indexed_list!(
    create Cars on Car using {
        id: fast_forward::index::uint::UIntIndex => 0
    }
);

fn main() {
    let cars = Cars::owned(vec![
        Car(2, "BMW".into()),
        Car(5, "Audi".into()),
        Car(2, "VW".into()),
        Car(99, "Porsche".into()),
    ]);

    assert!(cars.id().contains(&2));

    let r = cars.id().get(&2).collect::<Vec<_>>();
    assert_eq!(vec![&Car(2, "BMW".into()), &Car(2, "VW".into())], r);

    let mut it = cars.id().get(&5);
    assert_eq!(it.next(), Some(&Car(5, "Audi".into())));
    assert_eq!(it.next(), None);

    let mut it = cars.id().filter(|f| f.eq(&5));
    assert_eq!(it.next(), Some(&Car(5, "Audi".into())));
    assert_eq!(it.next(), None);

    let mut it = cars.id().get(&1000);
    assert_eq!(it.next(), None);

    assert_eq!(2, cars.id().meta().min());
    assert_eq!(99, cars.id().meta().max());
}
