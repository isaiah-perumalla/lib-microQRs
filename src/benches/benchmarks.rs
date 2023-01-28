#![feature(test)]
mod bench {
    extern crate test;

    use test::Bencher;
    use tiny_qr::codec::QrCode;
    use tiny_qr::error_cc::ErrorLevel;
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

    #[bench]
    fn bench_bytes_to_qrcode_v5(b: &mut Bencher) {
        // let inv = tiny_qr::error_correction::gf256::get_inverse(2);
        const VERSION: u8 = 5;
        let data_str =  "Unless the Lord builds the house,the builders labor in vain. Psalm-127 www.biblegateway.com/passage";

        b.iter(|| {
            let mut qr = QrCode::new(VERSION, ErrorLevel::L);
            test::black_box(&mut qr);

            qr.encode_data(data_str);
            test::black_box(qr);
        });
    }

    #[bench]
    fn bench_code_bytes_to_qrcode_v5(b: &mut Bencher) {
        let data_str =  "Unless the Lord builds the house,the builders labor in vain. Psalm-127 www.biblegateway.com/passage";
        b.iter(|| {
            for _ in 0..100 {
                let result = test::black_box(tiny_qr::encode::<144>(data_str));
                if let Ok(code) = result {
                    test::black_box(&code);
                } else {
                    panic!("err on encode");
                }
            }
        });
    }
}
