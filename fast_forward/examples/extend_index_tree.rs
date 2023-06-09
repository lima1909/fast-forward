use fast_forward::{
    collections::{Filter, OneIndexList},
    index::{store::Filterable, uint::UIntIndex},
    index::{IndexFilter, SelectedIndices, Store},
};

trait Parents<'f> {
    fn parents(&self, key: usize, stop: usize) -> SelectedIndices<'f>;
}

impl<'f, F, L> Parents<'f> for Filter<'f, F, L>
where
    F: Filterable<Key = usize>,
    L: IndexFilter<Item = Node>,
{
    fn parents(&self, key: usize, stop: usize) -> SelectedIndices<'f> {
        let mut result = SelectedIndices::empty();

        if key == stop {
            return result;
        }

        for i in self.eq(&key).iter() {
            let n = self.get(*i);
            result = self.eq(&n.parent) | self.parents(n.parent, stop);
        }

        result
    }
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
        nodes,
    );

    // PARENTS: up to the root node
    assert_eq!(None, n.idx().filter(|f| f.parents(9, 0)).next());
    assert_eq!(None, n.idx().filter(|f| f.parents(0, 0)).next());

    assert_eq!(
        vec![&Node::new(0, 0)],
        n.idx().filter(|f| f.parents(1, 0)).collect::<Vec<_>>()
    );
    assert_eq!(
        vec![&Node::new(0, 0)],
        n.idx().filter(|f| f.parents(4, 0)).collect::<Vec<_>>()
    );
    assert_eq!(
        vec![&Node::new(0, 0), &Node::new(1, 0)],
        n.idx().filter(|f| f.parents(2, 0)).collect::<Vec<_>>()
    );
    assert_eq!(
        vec![&Node::new(0, 0), &Node::new(1, 0)],
        n.idx().filter(|f| f.parents(3, 0)).collect::<Vec<_>>()
    );
    assert_eq!(
        vec![&Node::new(0, 0), &Node::new(1, 0), &Node::new(2, 1)],
        n.idx().filter(|f| f.parents(5, 0)).collect::<Vec<_>>()
    );
    assert_eq!(
        vec![
            &Node::new(0, 0),
            &Node::new(1, 0),
            &Node::new(2, 1),
            &Node::new(5, 2)
        ],
        n.idx().filter(|f| f.parents(6, 0)).collect::<Vec<_>>()
    );

    // // PARENTS-SUBTREE: NOT up to the root node
    assert_eq!(
        vec![&Node::new(2, 1), &Node::new(5, 2)],
        n.idx().filter(|f| f.parents(6, 2)).collect::<Vec<_>>()
    );
}
