use fast_forward_derive::Indexed;

#[derive(Indexed)]
#[index(core::clone::Clone)]
pub struct First {
    pub id: i32,
    pub name: String,
}

fn main() {
    let _f = First {
        id: 1,
        name: "Me".into(),
    };

    // println!("------- {}", f.foo());
}
