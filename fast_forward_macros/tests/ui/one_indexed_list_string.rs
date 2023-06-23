use fast_forward_macros::create_indexed_list;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Car(usize, String);

create_indexed_list!(
    create Cars on Car using {
        name: fast_forward::index::map::MapIndex => 1.to_lowercase
    }
);

fn main() {
    let cars = Cars::owned(vec![
        Car(2, "BMW".into()),
        Car(5, "Audi".into()),
        Car(2, "VW".into()),
        Car(99, "Porsche".into()),
    ]);

    // simple equals filter
    let r: Vec<&Car> = cars.name().get(&"vw".into()).collect();
    assert_eq!(vec![&Car(2, "VW".into())], r);

    // many/iter equals filter
    let r: Vec<&Car> = cars
        .name()
        .get_many(["vw".into(), "audi".into(), "bmw".into()])
        .collect();
    assert_eq!(
        vec![
            &Car(2, "BMW".into()),
            &Car(5, "Audi".into()),
            &Car(2, "VW".into()),
        ],
        r
    );

    // or equals query
    let r: Vec<&Car> = cars
        .name()
        .filter(|f| f.eq(&"vw".into()) | f.eq(&"audi".into()))
        .collect();
    assert_eq!(vec![&Car(5, "Audi".into()), &Car(2, "VW".into())], r);

    // update one Car
    assert_eq!(
        None,
        cars.name().filter(|f| f.eq(&"mercedes".into())).next()
    );
}
