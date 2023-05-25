use fast_forward::{
    fast,
    index::{map::MapIndex, uint::UIntIndex},
};

#[derive(Debug, Eq, PartialEq)]
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
    let mut cars = fast!(Cars on Car {id: UIntIndex => id});
    cars.insert(Car::new(2, "BMW"));
    cars.insert(Car::new(5, "Audi"));
    cars.insert(Car::new(2, "VW"));
    cars.insert(Car::new(99, "Porsche"));

    let r = cars.id().get(&2).collect::<Vec<_>>();
    assert_eq!(vec![&Car::new(2, "BMW"), &Car::new(2, "VW")], r);

    let r = cars
        .id()
        .filter(|f| f.eq(&2) | f.eq(&100))
        .collect::<Vec<_>>();

    assert_eq!(vec![&Car::new(2, "BMW"), &Car::new(2, "VW")], r);

    // ------------------------------
    // With `Name` Index: StrMapIndex
    // ------------------------------
    let mut cars = fast!(Cars on Car {name: MapIndex => name.clone});
    cars.insert(Car::new(2, "BMW"));
    cars.insert(Car::new(5, "Audi"));
    cars.insert(Car::new(2, "VW"));
    cars.insert(Car::new(99, "Porsche"));

    let r: Vec<&Car> = cars.name().get(&"VW".into()).collect();
    assert_eq!(vec![&Car::new(2, "VW")], r);

    let r: Vec<&Car> = cars
        .name()
        .filter(|f| f.eq(&"VW".into()) | f.eq(&"Audi".into()))
        .collect();
    assert_eq!(vec![&Car::new(5, "Audi"), &Car::new(2, "VW")], r);
}
