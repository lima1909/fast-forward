use std::borrow::Cow;
use std::collections::BTreeSet;

use criterion::{criterion_group, criterion_main, Criterion};

use fast_forward::index::map::MapIndex;
use fast_forward::index::uint::UIntIndex;
use fast_forward::index::{Retriever, Store};
use fast_forward::query::{and, or, query};

const HOW_MUCH_PERSON: usize = 100_000;
const FIND_ID: usize = 1_001;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Person(usize, String);

struct Indices {
    pk: UIntIndex,
    name: MapIndex,
}

impl Indices {
    fn insert(&mut self, p: &Person, idx: usize) {
        self.pk.insert(p.0, idx);
        self.name.insert(p.1.clone(), idx);
    }
}

fn list_index(c: &mut Criterion) {
    // create search vector
    let v = create_person_vec();

    #[allow(non_snake_case)]
    let FIND_PERSON: Person = Person(FIND_ID, format!("Jasmin {FIND_ID}"));

    // create search index
    let mut idx = Indices {
        pk: UIntIndex::with_capacity(HOW_MUCH_PERSON),
        name: MapIndex::with_capacity(HOW_MUCH_PERSON),
    };

    for (i, p) in v.iter().enumerate() {
        idx.insert(p, i);
    }

    // group benchmark
    let mut group = c.benchmark_group("index");
    group.bench_function("ff: query pk", |b| {
        b.iter(|| {
            let i = query(idx.pk.get(&FIND_ID)).exec()[0];
            assert_eq!(&FIND_PERSON, &v[i]);
        })
    });
    group.bench_function("ff: filter pk", |b| {
        b.iter(|| {
            let i = idx.pk.get(&FIND_ID)[0];
            assert_eq!(&FIND_PERSON, &v[i]);
        })
    });

    group.bench_function("vec-iter: pk", |b| {
        b.iter(|| {
            let v: Vec<&Person> = v.iter().filter(|p| p.0 == FIND_ID).collect();
            assert_eq!(&FIND_PERSON, v[0]);
        })
    });

    group.bench_function("ff: pk and name", |b| {
        b.iter(|| {
            let i = query(idx.pk.get(&FIND_ID))
                .and(idx.name.get(&FIND_PERSON.1))
                .exec()[0];
            assert_eq!(&FIND_PERSON, &v[i]);
        })
    });
    group.bench_function("vec-iter: pk and name", |b| {
        b.iter(|| {
            let v: Vec<&Person> = v
                .iter()
                .filter(|p| p.0 == FIND_ID && &p.1 == &FIND_PERSON.1)
                .collect();
            assert_eq!(&FIND_PERSON, v[0]);
        })
    });

    group.finish();
}

fn bit_operation(c: &mut Criterion) {
    let mut lv = Vec::new();
    for i in 0..50 {
        lv.push(i);
    }

    let mut rv = Vec::new();
    for i in 25..75 {
        rv.push(i);
    }

    let lts = BTreeSet::from_iter(lv.iter());
    let rts = BTreeSet::from_iter(rv.iter());

    // group benchmark
    let mut group = c.benchmark_group("and_or");
    // group benchmark
    group.bench_function("treeset and", |b| {
        b.iter(|| {
            let r = &lts & &rts;
            assert_eq!(25, r.len());
        })
    });

    group.bench_function("treeset or", |b| {
        b.iter(|| {
            let r = &lts | &rts;
            assert_eq!(75, r.len());
        })
    });

    group.bench_function("slice and", |b| {
        b.iter(|| {
            assert_eq!(25, and(&lv, &rv).len());
        })
    });

    group.bench_function("slice or", |b| {
        b.iter(|| {
            assert_eq!(75, or(Cow::Borrowed(&lv), Cow::Borrowed(&rv)).len());
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
        v.push(Person(i, format!("Jasmin {i}")));
    }
    v
}
