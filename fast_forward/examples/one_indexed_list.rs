use fast_forward::{
    collections::rw::IList,
    index::{MapIndex, UniqueUIntIndex},
};

#[derive(Debug, Eq, PartialEq, Clone)]
struct Car {
    id: usize,
    name: String,
}

impl Car {
    fn new(id: usize, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
        }
    }
}

fn main() {
    // -------------------------
    // With `ID Index: UIntIndex
    // -------------------------
    let cars = vec![
        Car::new(2, "BMW"),
        Car::new(5, "Audi"),
        Car::new(2, "VW"),
        Car::new(99, "Porsche"),
    ];

    let cars = IList::<UniqueUIntIndex, _, _>::from_iter(|c: &Car| c.id, cars.clone().into_iter());

    let r = cars.idx().get(&2).collect::<Vec<_>>();
    assert_eq!(vec![&Car::new(2, "BMW"), &Car::new(2, "VW")], r);

    let r = cars
        .idx()
        .filter(|f| f.eq(&2) | f.eq(&100))
        .collect::<Vec<_>>();

    assert_eq!(vec![&Car::new(2, "BMW"), &Car::new(2, "VW")], r);

    assert_eq!(Some(2), cars.idx().meta().min_key_index());
    assert_eq!(Some(99), cars.idx().meta().max_key_index());

    // ------------------------------
    // With `Name` Index: StrMapIndex
    // ------------------------------
    let cars = vec![
        Car::new(2, "BMW"),
        Car::new(5, "Audi"),
        Car::new(2, "VW"),
        Car::new(99, "Porsche"),
    ];

    let cars = IList::<MapIndex, _, _>::from_iter(|c: &Car| c.name.clone(), cars.into_iter());

    let r: Vec<&Car> = cars.idx().get(&"VW".into()).collect();
    assert_eq!(vec![&Car::new(2, "VW")], r);

    let r: Vec<&Car> = cars
        .idx()
        .filter(|f| f.eq(&"VW".into()) | f.eq(&"Audi".into()))
        .collect();
    assert_eq!(vec![&Car::new(5, "Audi"), &Car::new(2, "VW")], r);
}
