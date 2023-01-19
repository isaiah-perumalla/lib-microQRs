#![feature(test)]
mod bench {
    extern crate test;

    use test::Bencher;
    use tiny_qr::gf256::{gen_polynomial, Poly};

    const SAMPLE_DATA: [u8; 108] = [
        69, 21, 71, 39, 87, 55, 66, 6, 150, 226, 7, 70, 134, 82, 4, 196, 245, 36, 66, 7, 118, 151,
        70, 130, 6, 22, 198, 194, 7, 150, 247, 82, 6, 134, 86, 23, 39, 66, 6, 198, 86, 22, 226, 6,
        230, 247, 66, 6, 246, 226, 7, 150, 247, 87, 34, 6, 247, 118, 226, 7, 86, 230, 70, 87, 39,
        55, 70, 22, 230, 70, 150, 230, 114, 5, 7, 38, 247, 102, 87, 38, 39, 50, 0, 236, 17, 236,
        17, 236, 17, 236, 17, 236, 17, 236, 17, 236, 17, 236, 17, 236, 17, 236, 17, 236, 17, 236,
        17, 236,
    ];
    #[bench]
    fn bench_error_codes(b: &mut Bencher) {
        // let inv = tiny_qr::error_correction::gf256::get_inverse(2);

        b.iter(|| {
            let data = test::black_box(Poly::from(107, &SAMPLE_DATA));
            let gen = test::black_box(gen_polynomial(26));
            let gen_poly = test::black_box(&gen);

            let remainder = data.div_remainder(gen_poly);
            test::black_box(remainder);
        });
    }
}
