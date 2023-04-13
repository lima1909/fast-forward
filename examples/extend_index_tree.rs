use fast_forward::{
    fast,
    index::{uint::UIntIndex, Equals},
    query::or,
    Idx, EMPTY_IDXS,
};

use std::{borrow::Cow, ops::Index};

trait Tree: Equals<usize> {
    fn parents<I>(&self, key: usize, stop: usize, nodes: &I) -> Cow<[Idx]>
    where
        I: Index<usize, Output = Node>,
    {
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
}

fn main() {
    let mut fast_nodes = fast!(Nodes on Node {id: UIntIndex => id});

    //     0
    //   1   4
    // 2   3
    // 5
    // 6
    fast_nodes.insert(Node::new(0, 0));
    fast_nodes.insert(Node::new(1, 0));
    fast_nodes.insert(Node::new(2, 1));
    fast_nodes.insert(Node::new(3, 1));
    fast_nodes.insert(Node::new(4, 0));
    fast_nodes.insert(Node::new(5, 2));
    fast_nodes.insert(Node::new(6, 5));

    // access to the `_items_` field is not so nice
    let nodes = &fast_nodes._items_;

    // PARENTS: up to the root node
    assert!(fast_nodes.id.parents(9, 0, nodes).is_empty());
    assert!(fast_nodes.id.parents(0, 0, nodes).is_empty());

    assert_eq!(&[0], &fast_nodes.id.parents(1, 0, nodes)[..]);
    assert_eq!(&[0], &fast_nodes.id.parents(4, 0, nodes)[..]);
    assert_eq!(&[0, 1], &fast_nodes.id.parents(2, 0, nodes)[..]);
    assert_eq!(&[0, 1], &fast_nodes.id.parents(3, 0, nodes)[..]);
    assert_eq!(&[0, 1, 2], &fast_nodes.id.parents(5, 0, nodes)[..]);
    assert_eq!(&[0, 1, 2, 5], &fast_nodes.id.parents(6, 0, nodes)[..]);

    // PARENTS-SUBTREE: NOT up to the root node
    assert_eq!(&[2, 5], &fast_nodes.id.parents(6, 2, nodes)[..]);
}
