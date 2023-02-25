use criterion::{black_box, criterion_group, criterion_main, Criterion};

use fast_forward::index::uint::UIntVecIndex;
use fast_forward::index::{Filter, Indices, Unique};
use fast_forward::ops;

const HOW_MUCH_PERSON: usize = 100_000;
const FIND_ID: usize = 1_001;
const FIND_PERSON: Person = Person(FIND_ID, "Jasmin");

#[derive(Debug, Clone, PartialEq, Eq)]
struct Person(usize, &'static str);

fn list_index(c: &mut Criterion) {
    // create search vector
    let v = create_person_vec();

    // create search index
    let uint_idx = UIntVecIndex::<Unique>::with_capacity(HOW_MUCH_PERSON);
    let mut idx = Indices::new();
    idx.add_idx("pk", Box::new(uint_idx), |p: &Person| p.0);

    for i in 0..=HOW_MUCH_PERSON {
        idx.insert(&Person(i, "Jasmin"), i).unwrap();
    }
    let idx = idx.get_idx("pk");

    // group benchmark
    let mut group = c.benchmark_group("index");
    group.bench_function("list_index", |b| {
        b.iter(|| {
            let i = idx.idx(Filter::new(ops::EQ, FIND_ID))[0];
            assert_eq!(&FIND_PERSON, &v[i]);
        })
    });

    group.bench_function("vector", |b| {
        b.iter(|| {
            assert_eq!(&FIND_PERSON, v.iter().find(|p| p.0 == FIND_ID).unwrap());
        })
    });

    group.bench_function("vector Idx", |b| {
        b.iter(|| {
            let i = black_box([FIND_ID][0]);
            assert_eq!(&FIND_PERSON, v.get(i).unwrap());
        })
    });

    group.bench_function("vector short", |b| {
        b.iter(|| {
            assert_eq!(&FIND_PERSON, v.get(FIND_ID).unwrap());
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

fn create_person_vec() -> Vec<Person> {
    let mut v = Vec::new();
    for i in 0..=HOW_MUCH_PERSON {
        v.push(Person(i, "Jasmin"));
    }
    v
}
