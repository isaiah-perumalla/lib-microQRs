use crate::{add_padding, encode_byte_segment};
use crate::bits::{BitSquare, MsbBitIter, Square};
use crate::error_cc::ErrorLevel;


pub type MaskFN = fn((u8, u8)) -> bool;
pub static MASK_FN: [fn((u8, u8)) -> bool; 4] = [
    |(x, y)| 0 == ((x + y) % 2),
    |(_, y)| 0 == (y % 2),
    |(x, _)| 0 == (x % 3),
    |(x, y)| 0 == ((x + y) % 3),
];



#[cfg(test)]
mod encoder_test {
    use crate::bits::Square;
    use crate::encode_byte_segment;
    use crate::error_cc::ErrorLevel;
    use std::collections::HashSet;
    use crate::{encode, Version, ZigzagIter};

    #[test]
    pub fn test_encode_to_bytes() {
        let mut out_bytess = [0u8; 64];
        let res = encode_byte_segment("isaiah", &mut out_bytess);
        assert_eq!(res.unwrap(), 8);
        let expected_bytes = [0x40, 0x66, 0x97, 0x36, 0x16, 0x96, 0x16, 0x80];
        assert_eq!(&expected_bytes, &out_bytess[0..expected_bytes.len()]);

        let res = encode_byte_segment("isaiah-perumalla", &mut out_bytess);
        assert_eq!(res.unwrap(), 18);
        let expected_bytes = [
            0x41, 0x06, 0x97, 0x36, 0x16, 0x96, 0x16, 0x82, 0xD7, 0x06, 0x57, 0x27, 0x56, 0xD6,
            0x16, 0xC6, 0xC6, 0x10,
        ];
        assert_eq!(&expected_bytes, &out_bytess[0..expected_bytes.len()]);
    }

    #[test]
    fn test_qr_data_module_iter_by_version() {
        for i in 2..=5 {

            let data_modules: Vec<(u8, u8)> = Version(i).data_region_iter()
                .filter(|p| Version(i).is_data_location(*p))
                .collect();

            let expected_data_square_count = expected_data_module_count(i);
            let reaminder_bits = 7;
            assert_eq!(
                data_modules.len(),
                expected_data_square_count,
                "version={}",
                i
            );
            assert_eq!(
                expected_data_square_count,
                reaminder_bits + ErrorLevel::L.total_words(i) * 8,
                "bit count mismatch for version={}",
                i
            );
        }
    }

    #[test]
    fn test_qr_data_module_iter() {
        let data_modules: Vec<(u8, u8)> = Version(1).data_region_iter()
            .filter(|p| Version(1).is_data_location(*p))
            .collect();
        println!("{:?}", &data_modules);
        let data_modules_set: HashSet<(u8, u8)> = data_modules.iter().copied().collect();
        assert_eq!(
            data_modules_set.len(),
            data_modules.len(),
            "duplicate positions detected"
        );
        for i in 0..8 {
            let pos = (7, i);
            assert_eq!(
                false,
                data_modules_set.contains(&pos),
                "separator (7,{}) should not be in data modules",
                i
            );
        }
        let top_left_square = Square::new(9, (0, 0)); //includes separator and format area

        let top_right_square = Square::new(9, (13, 0)); //includes separator and format area
        let bottom_left_square = Square::new(9, (0, 13)); //includes separator and format area

        let v = Version(1);
        //square above dark module
        assert_eq!(
            true,
            v.is_data_location((8, 12)),
            "should be data module ({},{})",
            8,
            12
        );
        assert_eq!(
            true,
            v.is_data_location((7, 12)),
            "should be data module ({},{})",
            7,
            12
        );
        assert_eq!(
            false,
            v.is_data_location((6, 12)),
            "should NOT be data module ({},{})",
            6,
            12
        );
        for i in 0..6 {
            assert_eq!(
                true,
                v.is_data_location((i, 12)),
                "should be data module ({},{})",
                i,
                12
            );
        }
        for point in &data_modules {
            assert_eq!(
                false,
                top_left_square.contains_point(*point),
                "{:?} is not a data module",
                *point
            );
            assert_eq!(
                false,
                top_right_square.contains_point(*point),
                "{:?} is not a data module",
                *point
            );
            assert_eq!(
                false,
                bottom_left_square.contains_point(*point),
                "{:?} is not a data module",
                *point
            );


        }
        for i in 0..8 {
            let pos = (7, i);
            assert_eq!(
                false,
                data_modules_set.contains(&pos),
                "separator (7,{}) should not be in data modules",
                i
            );
        }
        println!("{:?}", &data_modules);
        let expected_data_square_count = expected_data_module_count(1);
        assert_eq!(data_modules.len(), expected_data_square_count);
        // println!("len={},{:?}", data_modules.len(), data_modules);
        let it: Vec<(u8, u8)> = Version(1).data_region_iter().filter(|(x,_)| *x < 6).collect();
        let expected_pos = [
            (5u8, 9u8),
            (4, 9),
            (5, 10),
            (4, 10),
            (5, 11),
            (4, 11),
            (5, 12),
            (4, 12),
            (3, 12),
            (2, 12),
            (3, 11),
            (2, 11),
            (3, 10),
            (2, 10),
            (3, 9),
            (2, 9),
            (1, 9),
            (0, 9),
            (1, 10),
            (0, 10),
            (1, 11),
            (0, 11),
            (1, 12),
            (0, 12),
        ];
        assert_eq!(&expected_pos[0..], &it);

    }

    fn expected_data_module_count(version: u8) -> usize {
        debug_assert!(version <= 5, "not implemente for version > 5");
        let aligment_squre_bits = if version > 1 { 25 } else { 0 };
        let square_size = Version(version).square_size() as usize;
        let timing_sq = 2 * (square_size - 16) as usize;
        let expected_data_square_count = (square_size * square_size) -
            (3 * 49) //finding module
            - 45 // seperators
            - 30 // version info 15 * 2
            - 1 // dark module
            - timing_sq; //timing
        expected_data_square_count - aligment_squre_bits
    }

    #[test]
    fn test_basic_qr_level1() {

        let code = encode::<64>("isaiah-perumalla").unwrap();

        let mut bit_string = String::new();
        for ((x,y), bit) in code.data_module_iter() {
            if bit {
                bit_string.push('1');
            } else {
                bit_string.push('0');
            }
        }
        let expected_unmasked_str = "0100000100000110100101110011011000010110100101100001011010000010110101110000011001010111001001110101011011010110000101101100011011000110000100001110110001111001010111010110010000010111000001111110011010001111";
        assert_eq!(expected_unmasked_str.len(), bit_string.len());
        assert_eq!(expected_unmasked_str, bit_string);
    }

    #[test]
    fn test_basic_qr_level2() {
        let code = encode::<64>("isaiah-perumalla1/kingsgrove").unwrap();
        assert_eq!(code.version.0, 2);
        let data_bit_count = ErrorLevel::L.total_words(2) * 8;
        let mut bit_string = String::new();
        for (_, bit) in code.data_module_iter() {

                if bit {
                    bit_string.push('1');
                } else {
                    bit_string.push('0');
                }
            }
            let expected_unmasked_str = "0100000111000110100101110011011000010110100101100001011010000010110101110000011001010111001001110101011011010110000101101100011011000110000100110001001011110110101101101001011011100110011101110011011001110111001001101111011101100110010100001110110000010001111011000001000101011011100001111010111100111100110100011011100101010000000110000010100110100011";
            assert_eq!(expected_unmasked_str, bit_string);

    }

    #[test]
    fn test_basic_qr_level3() {
        let code = encode::<128>("isaiah-perumalla1/kingsgrove-0dweqweqw").unwrap();
        assert_eq!(code.version.0, 3);
        let data_bit_count = ErrorLevel::L.total_words(3) * 8;
        let mut bit_string = String::new();
        for (_, bit) in code.data_module_iter() {

            if bit {
                bit_string.push('1');
            } else {
                bit_string.push('0');
            }
        }
        let expected_unmasked_str = "01000010011001101001011100110110000101101001011000010110100000101101011100000110010101110010011101010110110101100001011011000110110001100001001100010010111101101011011010010110111001100111011100110110011101110010011011110111011001100101001011010011000001100100011101110110010101110001011101110110010101110001011101110000111011000001000111101100000100011110110000010001111011000001000111101100000100011110110000010001111011000001000111101100000011001001000110101000111000110010111111100010101011001010001011010011011100110011001001100100011110011011001110011110";
        assert_eq!(expected_unmasked_str, bit_string);
    }
}
