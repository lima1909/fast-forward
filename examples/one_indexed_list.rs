use fast_forward::{
    index::{map::MapIndex, uint::UIntIndex, Equals},
    query::query,
    IndexedList, OneIndexedList,
};

#[derive(Debug, Eq, PartialEq)]
struct Car(usize, String);

impl Car {
    fn id(&self) -> usize {
        self.0
    }

    fn name(&self) -> String {
        self.1.clone()
    }
}

fn main() {
    // -------------------------
    // With `ID Index: UIntIndex
    // -------------------------
    let mut l = OneIndexedList::new(Car::id, UIntIndex::default());
    l.insert(Car(2, "BMW".into()));
    l.insert(Car(5, "Audi".into()));
    l.insert(Car(2, "VW".into()));
    l.insert(Car(99, "Porsche".into()));

    let r = l.filter(l.eq(2)).collect::<Vec<_>>();
    assert_eq!(vec![&Car(2, "BMW".into()), &Car(2, "VW".into())], r);

    let r = l
        .filter(query(l.eq(2)).or(l.eq(100)).exec())
        .collect::<Vec<_>>();
    assert_eq!(vec![&Car(2, "BMW".into()), &Car(2, "VW".into())], r);

    // ------------------------------
    // With `Name` Index: StrMapIndex
    // ------------------------------
    let mut l = OneIndexedList::new(Car::name, MapIndex::default());
    l.insert(Car(2, "BMW".into()));
    l.insert(Car(5, "Audi".into()));
    l.insert(Car(2, "VW".into()));
    l.insert(Car(99, "Porsche".into()));

    let r: Vec<&Car> = l.filter(l.eq(&"VW".into())).collect();
    assert_eq!(vec![&Car(2, "VW".into())], r);

    let r: Vec<&Car> = l
        .filter(query(l.eq(&"VW".into())).or(l.eq(&"Audi".into())).exec())
        .collect();
    assert_eq!(vec![&Car(5, "Audi".into()), &Car(2, "VW".into())], r);
}
