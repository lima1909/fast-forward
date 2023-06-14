use criterion::{criterion_group, criterion_main, Criterion};

use fast_forward::collections::ro::ROIndexList;
use fast_forward::index::map::MapIndex;
use fast_forward::index::uint::UIntIndex;
use fast_forward::index::{Filterable, Store};

const HOW_MUCH_PERSON: usize = 100_000;
const FIND_ID: usize = 1_001;
const FIND_ID_2: usize = 1_501;

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
    #[allow(non_snake_case)]
    let FIND_PERSON_2: Person = Person(FIND_ID_2, format!("Jasmin {FIND_ID_2}"));

    // read only index list
    let ro_idx = ROIndexList::new(UIntIndex::with_capacity(v.len()), |p: &Person| p.0, &v);

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
    group.bench_function("ff: ro pk get (one)", |b| {
        b.iter(|| {
            let p = ro_idx.idx().get(&FIND_ID).next().unwrap();
            assert_eq!(&FIND_PERSON, p);
        })
    });

    group.bench_function("ff: ro pk get (two)", |b| {
        b.iter(|| {
            let p = ro_idx.idx().get(&FIND_ID).next().unwrap();
            assert_eq!(&FIND_PERSON, p);

            let p = ro_idx.idx().get(&FIND_ID_2).next().unwrap();
            assert_eq!(&FIND_PERSON_2, p);
        })
    });

    group.bench_function("ff: ro pk get_many_cb (callback - two)", |b| {
        b.iter(|| {
            ro_idx
                .idx()
                .get_many_cb([FIND_ID, FIND_ID_2], |k, mut items| match k {
                    &FIND_ID => assert_eq!(&FIND_PERSON, items.next().unwrap()),
                    &FIND_ID_2 => assert_eq!(&FIND_PERSON_2, items.next().unwrap()),
                    _ => unreachable!("invalid Key: {k}"),
                });
        })
    });

    group.bench_function("ff: ro pk get_many (two)", |b| {
        b.iter(|| {
            let mut it = ro_idx.idx().get_many([FIND_ID, FIND_ID_2]);
            assert_eq!(&FIND_PERSON, it.next().unwrap());
            assert_eq!(&FIND_PERSON_2, it.next().unwrap());
            assert_eq!(None, it.next());
        })
    });

    group.bench_function("vec-iter: pk (one)", |b| {
        b.iter(|| {
            let mut it = v.iter().filter(|p| p.0 == FIND_ID);
            assert_eq!(Some(&FIND_PERSON), it.next());
            assert_eq!(None, it.next());
        })
    });

    group.bench_function("vec-iter: pk (two)", |b| {
        b.iter(|| {
            let mut it = v.iter().filter(|p| p.0 == FIND_ID || p.0 == FIND_ID_2);
            assert_eq!(Some(&FIND_PERSON), it.next());
            assert_eq!(Some(&FIND_PERSON_2), it.next());
            assert_eq!(None, it.next());
        })
    });

    group.bench_function("ff: pk and name", |b| {
        b.iter(|| {
            let i = (idx.pk.get(&FIND_ID) & idx.name.get(&FIND_PERSON.1))[0];
            assert_eq!(&FIND_PERSON, &v[i]);
        })
    });

    // compare with iter on a Vec and filter
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

criterion_group! {
    name = list;
    config = Criterion::default().significance_level(0.1).sample_size(100);
    targets = list_index
}

criterion_main!(list);

fn create_person_vec() -> Vec<Person> {
    let mut v = Vec::new();
    for i in 0..=HOW_MUCH_PERSON {
        v.push(Person(i, format!("Jasmin {i}")));
    }
    v
}
