use std::collections::HashSet;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use fast_forward::index::uint::UIntVecIndex;
use fast_forward::index::{Indices, Unique};
use fast_forward::query::{BinOp, Queryable};
use fast_forward::{eq, Idx, Key};

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
    let mut idx = Indices::new("pk", |p: &Person| Key::Usize(p.0), uint_idx);

    for i in 0..=HOW_MUCH_PERSON {
        idx.insert(&Person(i, "Jasmin"), i).unwrap();
    }

    let q = idx.query_builder::<HashSet<Idx>>();

    // group benchmark
    let mut group = c.benchmark_group("index");
    group.bench_function("list_index", |b| {
        b.iter(|| {
            let i = q.query(eq("pk", FIND_ID)).exec()[0];
            assert_eq!(&FIND_PERSON, &v[i]);
        })
    });

    group.bench_function("vector", |b| {
        b.iter(|| {
            let v: Vec<&Person> = v.iter().filter(|p| p.0 == FIND_ID).collect();
            assert_eq!(&FIND_PERSON, v[0]);
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

fn bit_operation(c: &mut Criterion) {
    let mut v = Vec::new();
    for i in 0..50 {
        v.push(i);
    }

    let lbop = HashSet::<Idx>::from_idx(&v);
    let rbop = HashSet::<Idx>::from_idx(&v);

    // group benchmark
    let mut group = c.benchmark_group("bitop");
    group.bench_function("hashset", |b| {
        b.iter(|| {
            let r = lbop.and(&rbop);
            assert_eq!(50, r.len());
        })
    });

    group.bench_function("from_idx", |b| {
        b.iter(|| {
            let lbop = HashSet::<Idx>::from_idx(&v);
            let rbop = HashSet::<Idx>::from_idx(&v);
            let r = lbop.and(&rbop);
            assert_eq!(50, r.len());
        })
    });

    group.finish();
}

criterion_group! {
    name = list;
    config = Criterion::default().significance_level(0.1).sample_size(100);
    targets = list_index
}

criterion_group! {
    name = bitop;
    config = Criterion::default().significance_level(0.1).sample_size(100);
    targets = bit_operation
}

criterion_main!(list, bitop);

fn create_person_vec() -> Vec<Person> {
    let mut v = Vec::new();
    for i in 0..=HOW_MUCH_PERSON {
        v.push(Person(i, "Jasmin"));
    }
    v
}
