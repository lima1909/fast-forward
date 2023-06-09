#![allow(dead_code)]
use fast_forward::index::Filterable;
use fast_forward_derive::Indexed;

#[derive(Indexed)]
pub struct First {
    #[index(fast_forward::index::uint::UIntIndex)]
    #[index(name = "new_id")]
    pub id: i32,
    pub name: String,
}

#[derive(Indexed)]
pub struct Second(
    #[index(fast_forward::index::uint::UIntIndex)]
    #[index(name = "id")]
    i32,
    String,
);

fn main() {
    let _f = First {
        id: 1,
        name: "Me".into(),
    };

    let l = FirstList::default();
    let r = l.new_id.get(&5);
    println!("Result FirstList: {r:?}");
    assert!(r.is_empty());

    let l = SecondList::default();
    let r = l.id.get(&5);
    println!("Result SecondList: {r:?}");
    assert!(r.is_empty());
}
