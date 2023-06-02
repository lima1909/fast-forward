use fast_forward::{
    collections::OneIndexList,
    index::{
        store::{Filter, Filterable},
        uint::UIntIndex,
    },
    index::{SelectedIndices, Store},
};

use std::ops::Index;

fn parents<'f, I, F>(f: &Filter<'f, F>, key: usize, stop: usize, nodes: &I) -> SelectedIndices<'f>
where
    I: Index<usize, Output = Node>,
    F: Filterable<Key = usize>,
{
    let mut result = SelectedIndices::empty();

    if key == stop {
        return result;
    }

    for i in f.eq(&key).iter() {
        let n = &nodes[*i];
        result = f.eq(&n.parent) | parents(f, n.parent, stop, nodes);
    }

    result
}

#[derive(Debug, PartialEq, Eq, Clone)]
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
    //
    //         0
    //       1   4
    //     2   3
    //   5
    // 6
    let nodes = vec![
        Node::new(0, 0),
        Node::new(1, 0),
        Node::new(2, 1),
        Node::new(3, 1),
        Node::new(4, 0),
        Node::new(5, 2),
        Node::new(6, 5),
    ];

    let n = OneIndexList::from_vec(
        UIntIndex::with_capacity(nodes.len()),
        |n: &Node| n.id,
        nodes.clone(),
    );

    // PARENTS: up to the root node
    assert_eq!(None, n.idx().filter(|f| parents(f, 9, 0, &nodes)).next());
    assert_eq!(None, n.idx().filter(|f| parents(f, 0, 0, &nodes)).next());

    assert_eq!(
        vec![&Node::new(0, 0)],
        n.idx()
            .filter(|f| parents(f, 1, 0, &nodes))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        vec![&Node::new(0, 0)],
        n.idx()
            .filter(|f| parents(f, 4, 0, &nodes))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        vec![&Node::new(0, 0), &Node::new(1, 0)],
        n.idx()
            .filter(|f| parents(f, 2, 0, &nodes))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        vec![&Node::new(0, 0), &Node::new(1, 0)],
        n.idx()
            .filter(|f| parents(f, 3, 0, &nodes))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        vec![&Node::new(0, 0), &Node::new(1, 0), &Node::new(2, 0)],
        n.idx()
            .filter(|f| parents(f, 5, 0, &nodes))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        vec![
            &Node::new(0, 0),
            &Node::new(1, 0),
            &Node::new(2, 0),
            &Node::new(5, 0)
        ],
        n.idx()
            .filter(|f| parents(f, 6, 0, &nodes))
            .collect::<Vec<_>>()
    );

    // // PARENTS-SUBTREE: NOT up to the root node
    assert_eq!(
        vec![&Node::new(2, 0), &Node::new(5, 0)],
        n.idx()
            .filter(|f| parents(f, 6, 2, &nodes))
            .collect::<Vec<_>>()
    );
}
