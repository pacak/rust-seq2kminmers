// based on https://raw.githubusercontent.com/luizirber/nthash/latest/benches/nthash.rs written by Luiz Irber
// but significantly extended. Also upgraded criterion from 0.2 to 0.4, different syntax.
//
// Still benchmarks the original crate, & extended to benchmark this crate too, and also.. the C++ NtHash2 version!

#[macro_use]
extern crate criterion;
extern crate bencher;

use rust_seq2kminmers::{FH,H};
use criterion::{Bencher, Criterion, Throughput, BenchmarkId};
use rand::distributions::{Distribution, Uniform};

use nthash::{nthash, NtHashIterator};
use nthash32;
#[allow(unused_imports)]
use rust_seq2kminmers::{KminmersIterator, KminmerType, KminmerVec, NtHashHPCIterator, NtHashSIMDIterator, HashMode};
//use rust_seq2kminmers::{nthash_c};

fn nthash_bench(c: &mut Criterion) {
    let range = Uniform::from(0..4);
    let mut rng = rand::thread_rng();
    let seq_len = 10000;
    let seq = (0..seq_len)
        .map(|_| match range.sample(&mut rng) {
            0 => 'A',
            1 => 'C',
            2 => 'G',
            3 => 'T',
            _ => 'N',
        })
        .collect::<String>();

    let mut group = c.benchmark_group("BenchmarkGroup");
    group.throughput(Throughput::Bytes(seq.len() as u64));

    group.bench_with_input(BenchmarkId::new("hpc_plain", seq_len), &seq, |b: &mut Bencher, i: &String| { b.iter(|| {
        let _hpc_str = rust_seq2kminmers::hpc(i);
        bencher::black_box(_hpc_str);
    })});

    group.bench_with_input(BenchmarkId::new("hpc_encode_rle", seq_len), &seq, |b: &mut Bencher, i: &String| { b.iter(|| {
        let _hpc_str = rust_seq2kminmers::encode_rle(i);
        bencher::black_box(_hpc_str);
    })});

    group.bench_with_input(BenchmarkId::new("hpc_encode_rle_simd", seq_len), &seq, |b: &mut Bencher, i: &String| { b.iter(|| {
        let _hpc_str = rust_seq2kminmers::encode_rle_simd(i.as_bytes());
        bencher::black_box(_hpc_str);
    })});

    group.bench_with_input(BenchmarkId::new("nthash (64)", seq_len), &seq,
    |b: &mut Bencher, i: &String| {
        b.iter(|| {
            nthash(i.as_bytes(), 5);
        })});

    group.bench_with_input(BenchmarkId::new("nthashiterator_luiz", seq_len), &seq,
    |b: &mut Bencher, i: &String| {
        b.iter(|| {
            let iter = NtHashIterator::new(i.as_bytes(), 5).unwrap();
            //  iter.for_each(drop);
            let _res = iter.collect::<Vec<u64>>(); // original nthash iterator only has 64 bits
            bencher::black_box(_res);
        })});

    group.bench_with_input(BenchmarkId::new("nthash32iterator_luiz", seq_len), &seq,
    |b: &mut Bencher, i: &String| {
        b.iter(|| {
            let iter = nthash32::NtHashIterator::new(i.as_bytes(), 5).unwrap();
            //  iter.for_each(drop);
            let _res = iter.collect::<Vec<u32>>(); // original nthash iterator only has 64 bits
            bencher::black_box(_res);
        })});

    group.bench_with_input(BenchmarkId::new("nthashiterator_hpc", seq_len), &seq,
    |b: &mut Bencher, i: &String| {
        b.iter(|| {
            let density : f64 = 0.01;
            let hash_bound = ((density as FH) * (H::max_value() as FH)) as H;
            let iter = NtHashHPCIterator::new(i.as_bytes(), 5, hash_bound).unwrap();
            let _res = iter.collect::<Vec<(usize, H)>>();
            bencher::black_box(_res);
        })});

    group.bench_with_input(BenchmarkId::new("nthashiterator_simd", seq_len), &seq,
    |b: &mut Bencher, i: &String| {
        b.iter(|| {
            let density : f64 = 0.01;
            let hash_bound = ((density as FH) * (H::max_value() as FH)) as H;
            let iter = NtHashSIMDIterator::new(i.as_bytes(), 5, hash_bound);
            let _res = iter.collect::<Vec<(usize, H)>>();
            bencher::black_box(_res);
        })});

    group.bench_with_input(BenchmarkId::new("kminmers_simd", seq_len), &seq, |b: &mut Bencher, i: &String| { b.iter(|| {
        let iter = KminmersIterator::new(i.as_bytes(), 10, 5, 0.01, HashMode::Simd).unwrap();
        let _res = iter.collect::<Vec<KminmerType>>();
        bencher::black_box(_res);
    })});


    group.bench_with_input(BenchmarkId::new("kminmers", seq_len), &seq,
    |b: &mut Bencher, i: &String| {
        b.iter(|| {
            let iter = KminmersIterator::new(i.as_bytes(), 10, 5, 0.01, HashMode::Regular).unwrap();
            let _res = iter.collect::<Vec<KminmerType>>();
            bencher::black_box(_res);
        })});

    group.bench_with_input(BenchmarkId::new("kminmers_hpc", seq_len), &seq,
    |b: &mut Bencher, i: &String| {
        b.iter(|| {
            let iter = KminmersIterator::new(i.as_bytes(), 10, 5, 0.01, HashMode::Hpc).unwrap();
            let _res = iter.collect::<Vec<KminmerType>>();
            bencher::black_box(_res);
        })});

    // The following bench requires to have compiled ntHash-C (https://github.com/rchikhi/ntHash-C)
    // Then uncomment those lines below and run with:
    /*
       LD_LIBRARY_PATH=path_to_ntHash-C/ \
       RUSTFLAGS='-Lpath_to_ntHash-C/ -lnthashc'  cargo bench \
       nthash_c_simple
    */
    // Spoiler: with this FFI integration, it's around the same speed as the rust implementation
    // even abit slower
    /*
    group.bench_with_input(BenchmarkId::new("nthash_c_simple", seq_len), &seq,
       |b: &mut Bencher, i: &String| {
        b.iter(|| {
            nthash_c(i.as_bytes(),5);
        })});
    
    */
    group.finish();
}

criterion_group!(benches, nthash_bench);
criterion_main!(benches);
