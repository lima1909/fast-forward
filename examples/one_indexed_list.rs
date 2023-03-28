use fast_forward::{
    index::{map::StrMapIndex, uint::UIntIndex},
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
    l.push(Car(2, "BMW".into()));
    l.push(Car(5, "Audi".into()));
    l.push(Car(2, "VW".into()));
    l.push(Car(99, "Porsche".into()));

    let r = l.filter(l.eq(2));
    assert_eq!(&[&Car(2, "BMW".into()), &Car(2, "VW".into())], &r[..]);

    let r = l.filter(query(l.eq(2)).or(l.eq(100)).exec());
    assert_eq!(&[&Car(2, "BMW".into()), &Car(2, "VW".into())], &r[..]);

    // ------------------------------
    // With `Name` Index: StrMapIndex
    // ------------------------------
    let mut l = OneIndexedList::new(Car::name, StrMapIndex::default());
    l.push(Car(2, "BMW".into()));
    l.push(Car(5, "Audi".into()));
    l.push(Car(2, "VW".into()));
    l.push(Car(99, "Porsche".into()));

    let r = l.filter(l.eq("VW"));
    assert_eq!(&[&Car(2, "VW".into())], &r[..]);

    let r = l.filter(query(l.eq("VW")).or(l.eq("Audi")).exec());
    assert_eq!(&[&Car(5, "Audi".into()), &Car(2, "VW".into())], &r[..])
}
