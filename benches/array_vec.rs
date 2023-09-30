//! Compare constructing an array vs Vec.

use std::iter;
use std::mem::{self, MaybeUninit};

use bytes::Bytes;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

const MSG_SIZE: usize = 1200;
const BATCH_SIZE: usize = 64;

fn array(c: &mut Criterion) {
    let payload: Vec<u8> = iter::repeat(1u8).take(MSG_SIZE).collect();
    let payload = Bytes::from(payload);
    c.bench_function("array", |b| {
        b.iter(|| {
            let payloads: [Bytes; BATCH_SIZE] = {
                let mut payloads: [MaybeUninit<Bytes>; BATCH_SIZE] =
                    unsafe { MaybeUninit::uninit().assume_init() };
                for elem in payloads.iter_mut() {
                    elem.write(payload.clone());
                }
                unsafe { mem::transmute(payloads) }
            };
            let _ = black_box(payloads);
        })
    });
}

fn vec(c: &mut Criterion) {
    let payload: Vec<u8> = iter::repeat(1u8).take(MSG_SIZE).collect();
    let payload = Bytes::from(payload);
    c.bench_function("vec", |b| {
        b.iter(|| {
            let payloads: Vec<Bytes> = iter::repeat_with(|| payload.clone())
                .take(BATCH_SIZE)
                .collect();
            let _ = black_box(payloads);
        })
    });
}

fn reused_vec(c: &mut Criterion) {
    let payload: Vec<u8> = iter::repeat(1u8).take(MSG_SIZE).collect();
    let payload = Bytes::from(payload);
    let mut payloads: Vec<Bytes> = Vec::with_capacity(BATCH_SIZE);
    c.bench_function("reused_vec", |b| {
        b.iter(|| {
            payloads.clear();
            payloads.extend(iter::repeat_with(|| payload.clone()).take(BATCH_SIZE));
        })
    });
}

criterion_group!(benches, array, vec, reused_vec);
criterion_main!(benches);
