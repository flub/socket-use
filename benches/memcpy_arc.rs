//! Compare cloning `Vec<u8>` vs `Bytes`.

use std::iter;

use bytes::Bytes;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

const BUF_SIZE: usize = 1200;

fn memcpy(c: &mut Criterion) {
    let buf: Vec<u8> = iter::repeat(1u8).take(BUF_SIZE).collect();
    c.bench_function("memcpy", |b| {
        b.iter(|| {
            let _ = black_box(buf.clone());
        })
    });
}

fn arc(c: &mut Criterion) {
    let buf: Vec<u8> = iter::repeat(1u8).take(BUF_SIZE).collect();
    let buf = Bytes::from(buf);
    c.bench_function("arc", |b| {
        b.iter(|| {
            let _ = black_box(buf.clone());
        })
    });
}

criterion_group!(benches, memcpy, arc);
criterion_main!(benches);
