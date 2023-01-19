use crate::gf256;
use crate::gf256::{gen_polynomial, Term};

#[derive(Clone, Copy, Debug)]
struct DataCapacity {
    ec_words_per_blk: u8,
    grp_1_blks: u8,
    words_per_grp_1: u8,
    grp_2_blks: u8,
    words_per_grp_2: u8,
}

impl DataCapacity {
    fn total_data_words(&self) -> usize {
        let words_grp_1 = self.words_per_grp_1 as u16 * (self.grp_1_blks as u16);
        let words_grp_2 = self.words_per_grp_2 as u16 * (self.grp_2_blks as u16);
        (words_grp_1 + words_grp_2) as usize
    }
}

const DATA_CAPACITY_L: [DataCapacity; 8] = [
    DataCapacity {
        ec_words_per_blk: 0,
        grp_1_blks: 0,
        words_per_grp_1: 0,
        grp_2_blks: 0,
        words_per_grp_2: 0,
    },
    DataCapacity {
        ec_words_per_blk: 7,
        grp_1_blks: 1,
        words_per_grp_1: 19,
        grp_2_blks: 0,
        words_per_grp_2: 0,
    }, //v1
    DataCapacity {
        ec_words_per_blk: 10,
        grp_1_blks: 1,
        words_per_grp_1: 34,
        grp_2_blks: 0,
        words_per_grp_2: 0,
    },
    DataCapacity {
        ec_words_per_blk: 15,
        grp_1_blks: 1,
        words_per_grp_1: 55,
        grp_2_blks: 0,
        words_per_grp_2: 0,
    },
    DataCapacity {
        ec_words_per_blk: 20,
        grp_1_blks: 1,
        words_per_grp_1: 80,
        grp_2_blks: 0,
        words_per_grp_2: 0,
    }, //v4
    DataCapacity {
        ec_words_per_blk: 26,
        grp_1_blks: 1,
        words_per_grp_1: 108,
        grp_2_blks: 0,
        words_per_grp_2: 0,
    }, //v5
    DataCapacity {
        ec_words_per_blk: 18,
        grp_1_blks: 2,
        words_per_grp_1: 68,
        grp_2_blks: 0,
        words_per_grp_2: 0,
    }, //v6
    DataCapacity {
        ec_words_per_blk: 20,
        grp_1_blks: 2,
        words_per_grp_1: 78,
        grp_2_blks: 0,
        words_per_grp_2: 0,
    }, //v7
];

#[derive(Clone, Copy)]
pub enum ErrorLevel {
    L,
    M,
    Q,
    H,
}

impl ErrorLevel {
    pub fn format_bits(&self, mask: u8) -> u32 {
        let l_mask_pattern: [u32; 8] = [
            0b111011111000100,
            0b111001011110011,
            0b111110110101010,
            0b111100010011101,
            0b110011000101111,
            0b110001100011000,
            0b110110001000001,
            0b110100101110110,
        ];
        match (*self, mask) {
            (ErrorLevel::L, m) => l_mask_pattern[m as usize],
            _ => 0,
        }
    }

    fn get_ecc_gf_poly(&self, version: u8) -> gf256::Poly {
        let ecc_size = DATA_CAPACITY_L[version as usize].ec_words_per_blk;
        gen_polynomial(ecc_size)
    }

    fn data_info(&self, version: u8) -> DataCapacity {
        match self {
            ErrorLevel::L => DATA_CAPACITY_L[version as usize],
            _ => todo!(),
        }
    }
    pub fn compute_ecc(&self, version: u8, block_data: &[u8], ecc_buffer: &mut [u8]) -> usize {
        let data_capacity_info = self.data_info(version);
        let ecc_size = data_capacity_info.ec_words_per_blk;
        let mut data_poly = gf256::Poly::from((block_data.len() - 1) as u8, block_data);
        let divisor = self.get_ecc_gf_poly(version);
        let data_p = data_poly.multiply(Term(divisor.degree, 1));
        let remainder = data_p.div_remainder(&divisor);
        for (i, term) in remainder.terms().enumerate() {
            ecc_buffer[i] = term.coef();
        }
        // debug_assert!(ecc_size, (remainder. degree + 1));
        return (remainder.degree + 1) as usize;
    }

    pub fn add_error_codes(&self, version: u8, msg_buffer: &mut [u8]) -> usize {
        match *self {
            ErrorLevel::L => {
                let capacity_info = DATA_CAPACITY_L[version as usize];
                let data_size = capacity_info.total_data_words();
                let mut ecc_words = [0u8; 32];
                let ecc_size =
                    ErrorLevel::L.compute_ecc(version, &msg_buffer[0..data_size], &mut ecc_words);
                let ecc_blk = &ecc_words[0..ecc_size];
                debug_assert!(
                    ecc_size == capacity_info.ec_words_per_blk as usize,
                    "ecc words per blk did not match "
                );
                for (i, byte) in ecc_blk.iter().enumerate() {
                    msg_buffer[data_size + i] = *byte;
                }
                (data_size + ecc_blk.len()) as usize
            }
            _ => todo!(),
        }
    }

    pub fn total_words(&self, v: u8) -> usize {
        match self {
            ErrorLevel::L => {
                let v = v as usize;
                DATA_CAPACITY_L[v].total_data_words() + DATA_CAPACITY_L[v].ec_words_per_blk as usize
            }
            _ => todo!(),
        }
    }

    pub fn data_code_words(&self, version: u8) -> usize {
        match self {
            ErrorLevel::L => DATA_CAPACITY_L[version as usize].total_data_words(),
            _ => todo!(),
        }
    }
}

#[cfg(test)]
mod ecc_tests {
    use crate::error_cc::ErrorLevel;
    use crate::gf256::gf_tests::hex_str_to_bytes;

    #[test]
    fn test_error_correction() {
        let data = hex_str_to_bytes("40 D4 A4 55 35 55 32 06 96 E2 04 B4 94 E4 70 EC 11 EC 11");
        let mut ecc_words = [0; 16];
        let version = 1;
        let ecc_size = ErrorLevel::L.compute_ecc(version, &data, &mut ecc_words);
        assert_eq!(ecc_size, 7);
        let expected_ecc = hex_str_to_bytes("31 CA A6 14 0E 5E EC");
        assert_eq!(&expected_ecc, &ecc_words[0..ecc_size]);
    }
}
