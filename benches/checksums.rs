use blake2::digest::{typenum::U4, Digest};
use divan::{counter::BytesCount, Bencher};

fn main() {
    // Run registered benchmarks.
    divan::main();
}

const SIZES: &[usize] = &[1 << 12, 1 << 14, 1 << 16, 1 << 20];
const LARGE_PAGE: &[u8; 1 << 20] = &{
    let mut page = [0; 1 << 20];
    let mut i = 0;
    while i < (1 << 20) {
        page[i] = i as u8;
        i += 1;
    }
    page
};

#[divan::bench(consts = SIZES)]
fn crc<const N: usize>(bencher: Bencher) {
    bencher
        .counter(BytesCount::new(N))
        .bench(|| crc32fast::hash(&divan::black_box(LARGE_PAGE)[..N]))
}

#[divan::bench(consts = SIZES)]
fn adler<const N: usize>(bencher: Bencher) {
    bencher
        .counter(BytesCount::new(N))
        .bench(|| adler::adler32_slice(&divan::black_box(LARGE_PAGE)[..N]))
}

#[divan::bench(consts = SIZES)]
fn blake2b_32<const N: usize>(bencher: Bencher) {
    bencher
        .counter(BytesCount::new(N))
        .bench(|| crypto::<blake2::Blake2b<U4>, N>())
}

#[divan::bench(consts = SIZES)]
fn blake2b_512<const N: usize>(bencher: Bencher) {
    bencher
        .counter(BytesCount::new(N))
        .bench(|| crypto::<blake2::Blake2b512, N>())
}

#[divan::bench(consts = SIZES)]
fn blake2s_32<const N: usize>(bencher: Bencher) {
    bencher
        .counter(BytesCount::new(N))
        .bench(|| crypto::<blake2::Blake2s<U4>, N>())
}

#[divan::bench(consts = SIZES)]
fn blake2s_256<const N: usize>(bencher: Bencher) {
    bencher
        .counter(BytesCount::new(N))
        .bench(|| crypto::<blake2::Blake2s256, N>())
}

#[divan::bench(consts = SIZES)]
fn blake3<const N: usize>(bencher: Bencher) {
    bencher.counter(BytesCount::new(N)).bench(|| {
        let res = blake3::hash(&divan::black_box(LARGE_PAGE)[..N]);
        u32::from_ne_bytes(res.as_bytes()[0..4].try_into().unwrap())
    })
}

#[divan::bench(consts = SIZES)]
fn sha1<const N: usize>(bencher: Bencher) {
    bencher
        .counter(BytesCount::new(N))
        .bench(|| crypto::<sha1::Sha1, N>())
}

#[divan::bench(consts = SIZES)]
fn sha256<const N: usize>(bencher: Bencher) {
    bencher
        .counter(BytesCount::new(N))
        .bench(|| crypto::<sha2::Sha256, N>())
}

fn crypto<D: Digest, const N: usize>() -> u32 {
    let res = D::new()
        .chain_update(&divan::black_box(LARGE_PAGE)[..N])
        .finalize();
    u32::from_ne_bytes(res[0..4].try_into().unwrap())
}
