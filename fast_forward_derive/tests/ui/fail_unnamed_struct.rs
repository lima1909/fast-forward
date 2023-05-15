use fast_forward_derive::Indexed;

#[derive(Indexed)]
pub struct Car(#[index(fast_forward::index::uint::UIntIndex)] i32, String);

fn main() {}
