use fast_forward_derive::Indexed;

#[derive(Indexed)]
pub struct Car(#[index(name = "id")] i32, String);

fn main() {}
