#![allow(dead_code)]
use fast_forward_derive::Indexed;

#[derive(Indexed)]
pub struct First {
    #[index(fast_forward::index::uint::UIntIndex)]
    pub id: i32,
    pub name: String,
}

fn main() {
    let _f = First {
        id: 1,
        name: "Me".into(),
    };

    let _l = FirstList::default();

    // let b = Bar::new(5);
    // b.foo(f);
    // println!("------- {}", f.foo());
}
