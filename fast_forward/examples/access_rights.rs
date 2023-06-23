#![allow(dead_code, unused_variables)]

use fast_forward::{collections::ROIndexList, index::uint::UIntIndex};

pub struct Confidential {
    id: usize,
    text: &'static str,
}

pub struct UserAccessRights<'a> {
    user_name: &'a str,
    read: ROIndexList<'a, usize, UIntIndex>,
}
impl<'a> UserAccessRights<'a> {
    fn new(user_name: &'a str, read: Vec<usize>) -> Self {
        Self {
            user_name,
            read: ROIndexList::owned(|id: &usize| *id, read),
        }
    }
    fn can_read(&self, secrets: &[Confidential]) {}
}

fn main() {
    let secrets = vec![
        Confidential {
            id: 99,
            text: "bla ...",
        },
        Confidential {
            id: 2043,
            text: "blub ...",
        },
    ];

    let access = UserAccessRights::new("me", vec![1, 3, 99]);
    access.can_read(&secrets);
}
