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
            #[divan::bench(consts = crate::SIZES)]
            fn $func_name<const N: usize>(bencher: divan::Bencher) {
                bencher
                    .counter(divan::counter::BytesCount::new(N))
                    .bench(|| ($func)(&divan::black_box(crate::LARGE_PAGE)[..N]))
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
    #[divan::bench_group]
    mod sha {
        use super::crypto;

        bench! {
            sha1 => crypto::<sha1::Sha1>;
            sha256 => crypto::<sha2::Sha256>;
        }
    }

    #[divan::bench_group]
    mod blake {
        use super::crypto;
        use blake2::digest::typenum::U4;

        bench! {
            blake2b_32 => crypto::<blake2::Blake2b<U4>>;
            blake2b_512 => crypto::<blake2::Blake2b512>;
            blake2s_32 => crypto::<blake2::Blake2s<U4>>;
            blake2s_256 => crypto::<blake2::Blake2s256>;
            blake3 => blake3_impl;
        }

        fn blake3_impl(page: &[u8]) -> u32 {
            let res = blake3::hash(page);
            u32::from_ne_bytes(res.as_bytes()[0..4].try_into().unwrap())
        }
    }

    fn crypto<D: blake2::digest::Digest>(page: &[u8]) -> u32 {
        let res = D::new().chain_update(page).finalize();
        u32::from_ne_bytes(res[0..4].try_into().unwrap())
    }
}
