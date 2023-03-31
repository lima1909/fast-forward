use fast_forward::{
    index::{uint::UIntIndex, Equals},
    query::or,
    Idx, OneIndexedList, EMPTY_IDXS,
};

use std::borrow::Cow;

trait Tree: Equals<usize> {
    fn parents(&self, key: usize, stop: usize, nodes: &[Node]) -> Cow<[Idx]> {
        let mut result = Cow::Borrowed(EMPTY_IDXS);

        if key == stop {
            return result;
        }

        for i in self.eq(key).iter() {
            let n = &nodes[*i];
            result = or(self.eq(n.parent), self.parents(n.parent, stop, nodes));
        }

        result
    }
}

impl Tree for UIntIndex {}

#[derive(Debug)]
struct Node {
    id: usize,
    parent: usize,
}

impl Node {
    fn new(id: usize, parent: usize) -> Self {
        Self { id, parent }
    }

    fn id(&self) -> usize {
        self.id
    }
}

fn main() {
    //     0
    //   1   4
    // 2   3
    // 5
    // 6
    let tree = vec![
        Node::new(0, 0),
        Node::new(1, 0),
        Node::new(2, 1),
        Node::new(3, 1),
        Node::new(4, 0),
        Node::new(5, 2),
        Node::new(6, 5),
    ];

    let mut nidxs = OneIndexedList::new(Node::id, UIntIndex::default());
    tree.into_iter().for_each(|n| nidxs.insert(n));

    let nodes: &[Node] = nidxs.as_ref();

    // PARENTS: up to the root node
    assert!(nidxs.parents(9, 0, nodes).is_empty());
    assert!(nidxs.parents(0, 0, nodes).is_empty());

    assert_eq!(&[0], &nidxs.parents(1, 0, nodes)[..]);
    assert_eq!(&[0], &nidxs.parents(4, 0, nodes)[..]);
    assert_eq!(&[0, 1], &nidxs.parents(2, 0, nodes)[..]);
    assert_eq!(&[0, 1], &nidxs.parents(3, 0, nodes)[..]);
    assert_eq!(&[0, 1, 2], &nidxs.parents(5, 0, nodes)[..]);
    assert_eq!(&[0, 1, 2, 5], &nidxs.parents(6, 0, nodes)[..]);

    // PARENTS-SUBTREE: NOT up to the root node
    assert_eq!(&[2, 5], &nidxs.parents(6, 2, nodes)[..]);
}
