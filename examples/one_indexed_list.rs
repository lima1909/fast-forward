use fast_forward::{
    fast,
    index::{map::MapIndex, uint::UIntIndex, Equals},
    query::query,
    IndexedList,
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
    let mut fast_cars = fast!(FastCars => Car {id: UIntIndex => id});
    fast_cars.insert(Car::new(2, "BMW"));
    fast_cars.insert(Car::new(5, "Audi"));
    fast_cars.insert(Car::new(2, "VW"));
    fast_cars.insert(Car::new(99, "Porsche"));

    let r = fast_cars.filter(fast_cars.id.eq(2)).collect::<Vec<_>>();
    assert_eq!(vec![&Car::new(2, "BMW"), &Car::new(2, "VW")], r);

    let r = fast_cars
        .filter(query(fast_cars.id.eq(2)).or(fast_cars.id.eq(100)).exec())
        .collect::<Vec<_>>();
    assert_eq!(vec![&Car::new(2, "BMW"), &Car::new(2, "VW")], r);

    // ------------------------------
    // With `Name` Index: StrMapIndex
    // ------------------------------
    let mut fast_cars = fast!(FastCars => Car {name: MapIndex => name.clone});
    fast_cars.insert(Car::new(2, "BMW"));
    fast_cars.insert(Car::new(5, "Audi"));
    fast_cars.insert(Car::new(2, "VW"));
    fast_cars.insert(Car::new(99, "Porsche"));

    let r: Vec<&Car> = fast_cars.filter(fast_cars.name.eq(&"VW".into())).collect();
    assert_eq!(vec![&Car::new(2, "VW")], r);

    let r: Vec<&Car> = fast_cars
        .filter(
            query(fast_cars.name.eq(&"VW".into()))
                .or(fast_cars.name.eq(&"Audi".into()))
                .exec(),
        )
        .collect();
    assert_eq!(vec![&Car::new(5, "Audi"), &Car::new(2, "VW")], r);
}
