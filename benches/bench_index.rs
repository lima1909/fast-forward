use criterion::{criterion_group, criterion_main, Criterion};
use fast_forward::index::Indices;
use fast_forward::index::{uint::U32Index, UniqueIdx};
use fast_forward::ops::eq;

const HOW_MUCH_PERSON: usize = 100_000;
const FIND_ID: usize = 1_001;

fn list_index(c: &mut Criterion) {
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Person(usize, &'static str);

    let mut idx = Indices::new();
    idx.add("pk", Box::<U32Index<UniqueIdx>>::default(), |p: &Person| {
        p.0
    });

    for i in 0..=HOW_MUCH_PERSON {
        idx.insert_index("pk", &Person(i, "Jasmin"), i).unwrap();
    }
    let idx = idx.store("pk");

    let mut v = Vec::new();
    for i in 0..=HOW_MUCH_PERSON {
        v.push(Person(i, "Jasmin"));
    }

    let mut group = c.benchmark_group("list_index-vs-vector");

    group.bench_function("list_index", |b| {
        b.iter(|| {
            assert_eq!(FIND_ID, idx.filter(eq(FIND_ID))[0]);
        })
    });

    group.bench_function("vector", |b| {
        b.iter(|| {
            assert_eq!(
                &Person(FIND_ID, "Jasmin"),
                v.iter().find(|p| p.0 == FIND_ID).unwrap()
            );
        })
    });

    group.finish();
}

criterion_group! {
    name = list;
    config = Criterion::default().significance_level(0.1).sample_size(100);
    targets = list_index
}

criterion_main!(list);
