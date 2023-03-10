use std::collections::HashSet;

use criterion::{criterion_group, criterion_main, Criterion};

use fast_forward::index::map::UniqueStrIdx;
use fast_forward::index::uint::UIntVecIndex;
use fast_forward::index::{And, Index, Indices, Multi, Or, Unique};
use fast_forward::query::{BinOp, Queryable};
use fast_forward::{eq, Idx, Key};

const HOW_MUCH_PERSON: usize = 100_000;
const FIND_ID: usize = 1_001;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Person(usize, String);

fn list_index(c: &mut Criterion) {
    // create search vector
    let v = create_person_vec();

    #[allow(non_snake_case)]
    let FIND_PERSON: Person = Person(FIND_ID, format!("Jasmin {FIND_ID}"));

    // create search index
    let mut idx = Indices::new(
        "pk",
        |p: &Person| Key::Usize(p.0),
        UIntVecIndex::<Unique>::with_capacity(HOW_MUCH_PERSON),
    );
    idx.add_idx("name", |p: &Person| Key::Str(&p.1), UniqueStrIdx::default());
    for (i, p) in v.iter().enumerate() {
        idx.insert(p, i).unwrap();
    }
    let q = idx.query_builder::<HashSet<Idx>>();

    // group benchmark
    let mut group = c.benchmark_group("index");
    group.bench_function("ff: pk", |b| {
        b.iter(|| {
            let i = q.query(eq("pk", FIND_ID)).exec().next().unwrap();
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
            let i = q
                .query(eq("pk", FIND_ID))
                .and(eq("name", &FIND_PERSON.1))
                .exec()
                .next()
                .unwrap();
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
    let mut multi_1 = Multi::new(0);
    let mut multi_2 = Multi::new(0);

    let mut v = Vec::new();
    for i in 0..50 {
        v.push(i);
        if i > 0 {
            multi_1.add(i).unwrap();
            multi_2.add(i).unwrap();
        }
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

    let lbop = roaring::RoaringBitmap::from_idx(&v);
    let rbop = roaring::RoaringBitmap::from_idx(&v);

    // group benchmark
    group.bench_function("roaring", |b| {
        b.iter(|| {
            let r = lbop.and(&rbop);
            assert_eq!(50, r.len());
        })
    });

    group.bench_function("multi and", |b| {
        b.iter(|| {
            let r = multi_1.and(multi_2.get()).unwrap();
            assert_eq!(50, r.len());
        })
    });

    group.bench_function("multi or", |b| {
        b.iter(|| {
            let r = multi_1.or(multi_2.get());
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
        v.push(Person(i, format!("Jasmin {i}")));
    }
    v
}
