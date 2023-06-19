use fast_forward_macro::create_indexed_list;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Car(usize, String);

// impl Car {
//     fn id(&self) -> usize {
//         self.0
//     }
// }

create_indexed_list!(create ro Cars);

fn main() {
    let cars = Cars;
    dbg!(cars);
}
