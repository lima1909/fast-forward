use fast_forward_derive::Indexed;

#[derive(Indexed)]
// #[index(core::clone::Clone)]
pub struct First {
    pub id: i32,
    pub name: String,
}

fn main() {
    let f = First {
        id: 1,
        name: "Me".into(),
    };

    let b = Bar::new(5);
    b.foo(f);
    // println!("------- {}", f.foo());
}
