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

macro_rules! bench {
    ($($func_name:ident => $func:expr;)*) => {
        $(
            #[divan::bench(args = crate::SIZES)]
            fn $func_name(bencher: divan::Bencher, n: usize) {
                bencher
                    .counter(divan::counter::BytesCount::new(n))
                    .bench(|| ($func)(&divan::black_box(crate::LARGE_PAGE)[..n]))
            }
        )*
    };
}

#[divan::bench_group]
mod non_crypto {
    bench! {
        crc => crc32fast::hash;
        adler => adler::adler32_slice;
    }
}

#[divan::bench_group]
mod crypto {
    use blake2::digest::{generic_array::GenericArray, Digest};

    #[divan::bench_group]
    mod md {
        bench! {
            md5 => md5_impl;
        }

        fn md5_impl(page: &[u8]) -> u128 {
            let res = md5::compute(page);
            u128::from_ne_bytes(res.0)
        }
    }

    #[divan::bench_group]
    mod sha {
        use super::crypto;

        bench! {
            sha1 => crypto::<sha1::Sha1>;
            sha2_256 => crypto::<sha2::Sha256>;
            sha3_256 => crypto::<sha3::Sha3_256>;
            sha2_384 => crypto::<sha2::Sha384>;
            sha3_384 => crypto::<sha3::Sha3_384>;
            sha2_512 => crypto::<sha2::Sha512>;
            sha3_512 => crypto::<sha3::Sha3_512>;
        }
    }

    #[divan::bench_group]
    mod blake {
        use super::crypto;

        bench! {
            blake2b => crypto::<blake2::Blake2b512>;
            blake2s => crypto::<blake2::Blake2s256>;
            blake3_mt => blake3_mt_impl;
            blake3 => blake3_impl;
        }

        fn blake3_mt_impl(page: &[u8]) -> [u8; 32] {
            let res = blake3::Hasher::new().update_rayon(page).finalize();
            *res.as_bytes()
        }

        fn blake3_impl(page: &[u8]) -> [u8; 32] {
            let res = blake3::Hasher::new().update(page).finalize();
            *res.as_bytes()
        }
    }

    fn crypto<D: Digest>(page: &[u8]) -> GenericArray<u8, D::OutputSize> {
        D::new().chain_update(page).finalize()
    }
}
