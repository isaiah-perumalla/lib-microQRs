

    const MAX_DEGREE:usize = 256;
    const GF256_INVERSE: [u8;256] = [0, 1, 142, 244, 71, 167, 122, 186, 173, 157, 221, 152, 61, 170,
        93, 150, 216, 114, 192, 88, 224, 62, 76, 102, 144, 222, 85, 128, 160, 131, 75, 42, 108, 237,
        57, 81, 96, 86, 44, 138, 112, 208, 31, 74, 38, 139, 51, 110, 72, 137, 111, 46, 164, 195, 64,
        94, 80, 34, 207, 169, 171, 12, 21, 225, 54, 95, 248, 213, 146, 78, 166, 4, 48, 136, 43, 30,
        22, 103, 69, 147, 56, 35, 104, 140, 129, 26, 37, 97, 19, 193, 203, 99, 151, 14, 55, 65, 36,
        87, 202, 91, 185, 196, 23, 77, 82, 141, 239, 179, 32, 236, 47, 50, 40, 209, 17, 217, 233, 251,
        218, 121, 219, 119, 6, 187, 132, 205, 254, 252, 27, 84, 161, 29, 124, 204, 228, 176, 73, 49,
        39, 45, 83, 105, 2, 245, 24, 223, 68, 79, 155, 188, 15, 92, 11, 220, 189, 148, 172, 9, 199,
        162, 28, 130, 159, 198, 52, 194, 70, 5, 206, 59, 13, 60, 156, 8, 190, 183, 135, 229, 238, 107,
        235, 242, 191, 175, 197, 100, 7, 123, 149, 154, 174, 182, 18, 89, 165, 53, 101, 184, 163, 158,
        210, 247, 98, 90, 133, 125, 168, 58, 41, 113, 200, 246, 249, 67, 215, 214, 16, 115, 118, 120,
        153, 10, 25, 145, 20, 63, 230, 240, 134, 177, 226, 241, 250, 116, 243, 180, 109, 33, 178, 106,
        227, 231, 181, 234, 3, 143, 211, 201, 66, 212, 232, 117, 127, 255, 126, 253];

    // static INIT_INV_TABLE: Once = Once::new();
    // static  mut GF_256_INV: [u8;256] = [0u8;256]
    const ZERO_POLY: Poly = Poly {degree: 0, cof: [0;MAX_DEGREE] };

    pub fn get_inverse(x:u8) -> u8 {
        GF256_INVERSE[x as usize]
        // unsafe {
        //     INIT_INV_TABLE.call_once(|| {
        //         GF_256_INV = compute_inv_table();
        //     });
        //     println!("{:?}", &GF_256_INV);
        //     GF_256_INV[x as usize]
        // }

    }

    fn compute_inv_table() -> [u8;256] {
        let mut inv = [0u8;256];
        inv[0] = 0;
        for x in 1..=255 {
            for y in 1..=255 {
                let res = gf256_mult(x, y);
                if res == 1 {
                    inv[x as usize] = y;
                    break;
                }
            }
        }
        #[cfg(debug_assertions)]
        check_inv_table(&inv);
        inv
    }

    #[cfg(debug_assertions)]
    fn check_inv_table(table: &[u8]) {
        debug_assert!(table.len() == 256);
        for i in 1..=255 {
            debug_assert!(gf256_mult(i, table[i as usize]) == 1);
        }
    }

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct Term (pub u8, pub u8); //represent polynomial term coffiecent, degree

    impl Term {

        pub fn div(&self, divisor: Term) -> Term {
            if divisor.degree() > self.degree() {
                return Term::zero();
            }
            let degree =  self.degree() - divisor.degree();
            let inv = get_inverse(divisor.coef());
            let coeff = gf256_mult(self.coef(), inv);
            Term(degree, coeff)
        }

        pub fn degree(&self) -> u8 {
            self.0
        }

        pub fn coef(&self) -> u8 {
            self.1
        }

        pub fn zero() -> Term {
            Term(0,0)
        }
     }


    #[derive(Clone)]
    pub struct Poly {
        pub degree: u8,
        pub cof: [u8; MAX_DEGREE]
    }


    impl Poly {
        pub fn from (degree: u8, coefficients: &[u8]) -> Poly {
            debug_assert!((degree + 1 )as usize == coefficients.len(), "degree must match coffecients-len");
            let mut cof = [0;MAX_DEGREE];
            for d in (0..=degree).rev() {
                let i = d as usize;
                if i < coefficients.len() {
                    let idx = degree as usize - i;
                    cof[i] = coefficients[idx];
                }

            }
            Poly {
                degree: degree as u8,
                cof
            }
        }

        pub fn leading_term(&self) -> Term {
            // debug_assert!(self.cof[self.degree] > 0, "leading cof cannot be zero");
            Term(self.degree, self.cof[self.degree as usize])
        }

        pub fn leading_cof(&self) -> u8 {
            self.leading_term().coef()
        }

        //highest degree coefficient at index 0
        pub fn coefficients(&self, result: &mut [u8]) -> u8 {
            for i in (0..=self.degree).rev() {
                let index = i as usize;
                result[(self.degree - i) as usize] = self.cof[index];
            }
            self.degree
        }

        //return remainder poly when divided by divisor
        pub fn div_remainder(&self, divisor: &Poly) -> Poly {
            let mut result =  self.clone(); //copy of self
            let divisor_term = divisor.leading_term();
            while !result.is_zero() && result.degree >= divisor.degree {
                let leading_term = result.leading_term();
                let d = leading_term.div(divisor_term);
                let poly = divisor.multiply(d);
                result.mut_add_poly(&poly);
            }
            debug_assert!(result.degree < divisor.degree, "remainder degree must be less than divisor");
            result
        }

        pub fn mut_mult_scalar(&mut self, scalar: u8) {
            let size = self.degree as usize;
            for i in 0..=size {
                let result = gf256_mult(scalar, self.cof[i]);
                self.cof[i as usize] = result;
            }
        }


        pub(crate) fn multiply(&self, term: Term) -> Poly {
            if term.coef() == 0 {
                return ZERO_POLY;
            }
            let mut result = self.clone(); //stack copy
            for i in 0..=result.degree {
                result.cof[i as usize] = gf256_mult(result.cof[i as usize], term.coef());
            }
            let new_degree = result.degree + term.degree();
            let shift = new_degree - result.degree;
            if shift > 0 {
                for i in (0..=result.degree).rev() {
                    let idx: usize = (i + shift) as usize;
                    result.cof[idx] = result.cof[i as usize];
                }
                for i in 0..shift {
                    result.cof[i as usize] = 0;
                }
            }
            result.degree = new_degree;
            result
        }
        fn is_zero(&self) -> bool {
            return self.cof.iter().all(|c| {*c == 0});
        }

        pub fn mut_add(&mut self, term: Term) {
            if term.degree() > self.degree {
                 ;
                debug_assert!((term.degree() as usize) < self.cof.len(),
                              "len exceeded term.degree={}, len={}", term.degree(), self.cof.len());
                self.cof[term.degree() as usize] = term.coef();
                for i in (self.degree + 1)..term.degree() {
                    self.cof[i as usize] = 0; //zeros in between if needed
                }
                self.degree = term.degree();
            } else {
                let term_degree = term.degree() as usize;
                self.cof[term_degree as usize] = gf256_add(self.cof[term_degree], term.coef()); //xor
            }
            let degree = self.degree;
            for i in (0..=degree).rev() {
                if self.cof[i as usize] == 0 {
                    self.degree -= 1;
                }
                else {
                    break;
                }
            }
        }

        pub fn mut_add_poly(&mut self, other: &Poly) {
            for term in other.terms() {
                self.mut_add(term);
            }
        }

        pub fn terms(&self) -> TermIter {
            TermIter::new(&self)
        }
    }

    pub struct TermIter<'a> {
        poly: &'a Poly,
        term_idx: u8
    }

    impl <'a> TermIter<'a> {
        pub fn  new(poly: &'a Poly) -> TermIter {

            TermIter {
                poly,
                term_idx: poly.degree + 1
            }
        }
    }

    impl <'a> Iterator for TermIter<'a> {
        type Item = Term;

        fn next(&mut self) -> Option<Self::Item> {

                if self.term_idx == 0  {
                    return None;
                }
                let index = (self.term_idx - 1) as usize;
                let coeff = self.poly.cof[index];
                self.term_idx -= 1;
                return Some(Term(index as u8, coeff));
        }
    }

    pub fn gen_polynomial(size: u8) -> Poly {
        let mut p = Poly::from(1, &[1, 1]);
        let mut α_i = 1;
        let X = Term(1, 1);
        for i in 1u8..size {
            let mut p_x_i = p.multiply(X);
            α_i = gf256_mult(α_i, 2u8); //α^i
            p.mut_mult_scalar(α_i);
            p_x_i.mut_add_poly(&p);
            p = p_x_i;
        }
        p
    }


    pub fn gf256_add(x: u8, y: u8) -> u8 {
        x ^ y
    }

    pub(crate) fn gf256_mult(x: u8, y: u8) -> u8 {
        //field prime  poly 0x11d with α generator 2
        //russian peasant multiplication using xor for add
        let mut x = x;
        let mut y = y as u16;
        let mut result = 0u16;
        while x > 0 {
            if (x & 1) == 1 { //odd
                result = result ^ y; //gf add xor
            }
            x = x >> 1;
            y = y << 1;
            if y > 255 { //cannot go outside gf256
                y = y ^ 0x11d;
            }
        }
        debug_assert!(result < 256);
        result as u8
    }


#[cfg(test)]
pub mod gf_tests {

    pub fn hex_str_to_bytes(str: &str) -> Vec<u8> {
        let bytes: Vec<u8> = str.split_ascii_whitespace().map(|s|  hex_byte(s)).collect();
        bytes
    }

    use crate::gf256::gen_polynomial;
    use crate::gf256::get_inverse;
    use crate::gf256::gf256_mult;
    use crate::gf256::Poly;
    use crate::gf256::Term;

    fn hex_byte(str: &str) -> u8 {
        debug_assert!(str.len() < 3 && str.len() > 0);
        let mut byte = 0u8;
        for (i,ch) in str.bytes().enumerate() {
            let shift = 4 * (1-i);
            match ch {
                b'0'..=b'9' =>  {
                    byte |= (ch - b'0') << shift;
                }
                b'a'..=b'f' => {
                    let v = 10 +  (ch - b'a');
                    byte |= v << shift;
                }
                b'A'..=b'F' => {
                    let v = 10 + (ch - b'A');
                    byte |= v << shift;
                }
                _ => panic!("not hex")
            }
        }
        byte
    }

    #[test]
    fn test_inverse_gf256() {
        assert_eq!(get_inverse(10), 221);
        assert_eq!(get_inverse(255), 253);

    }

    #[test]
    fn test_gf256_mult() {
        assert_eq!(29, gf256_mult(128, 2));
        assert_eq!(221,gf256_mult(68, 68));
        assert_eq!(0xee, gf256_mult(15, 18));
        assert_eq!(0x2b, gf256_mult(0x36, 0x12));
    }

    #[test]
    fn test_term_div() {
        // 4x^2
        assert_eq!(Term(2, 4).div(Term(1, 2)),
                   Term(1, 2));
        assert_eq!(Term(2, 4).div(Term(3, 2)),
                   Term(0, 0));
    }

    #[test]
    fn test_poly_add_term() {
        let mut poly = Poly::from(6, &[12, 0, 34, 64, 0, 0, 0]);
        poly.mut_add(Term(6, 10));
        let terms: Vec<Term> = poly.terms().collect();
        assert_eq!(&terms,  //xor add
                   &[Term(6, 6), Term(5, 0), Term(4, 34), Term(3, 64),
                       Term(2, 0), Term(1, 0), Term(0, 0)]);


        poly.mut_add(Term(7, 100));

        let terms: Vec<Term> = poly.terms().collect();
        assert_eq!(&terms,  //xor add
                   &[Term(7, 100), Term(6, 6), Term(5, 0), Term(4, 34),
                       Term(3, 64), Term(2, 0), Term(1, 0), Term(0, 0)]);

        poly.mut_add(Term(17, 117));

        let terms: Vec<Term> = poly.terms().filter(|t| {t.1 > 0}).collect();
        assert_eq!(poly.degree, 17);

        assert_eq!(&terms,  //xor add
                   &[Term(17, 117), Term(7, 100), Term(6, 6), Term(4, 34),
                       Term(3, 64)]);


        poly.mut_add(Term(17, 117));
        poly.mut_add(Term(7, 100));

        assert_eq!(poly.degree, 6);
        let terms: Vec<Term> = poly.terms().filter(|t| {t.coef() > 0}).collect();
        assert_eq!(&terms,  //xor add
                   &[Term(6, 6), Term(4, 34), Term(3, 64)]);

    }
        #[test]
    fn test_poly_mul_term() {
        let poly = Poly::from(6, &[12, 0, 34, 64, 0, 0, 0]);
        let result = poly.multiply(Term(0, 2)); // same as scalar 64
        assert_eq!(result.degree, 6);
            let terms: Vec<Term> = result.terms().filter(|t| {t.coef() > 0}).collect();
        assert_eq!(&terms,
                   &[Term(6, 24), Term(4, 68), Term(3, 128)]);

        let result = poly.multiply(Term(4, 2)); // same as scalar 64
        assert_eq!(result.degree, 10);
        let terms: Vec<Term> = result.terms().filter(|t| { t.coef() > 0 }).collect();
        assert_eq!(&terms,
                   &[Term(10, 24), Term(8, 68), Term(7, 128)]);
    }

    #[test]
    fn test_poly_gf_mult() {
        let mut poly = Poly::from(6, &[12, 34, 64, 0, 0, 0, 0]);
        poly.mut_mult_scalar(16);
        let mut result = [0u8;8];
        poly.coefficients(&mut result);
        assert_eq!(&[192, 26, 116, 0, 0, 0, 0], &result[0..7] );
    }


    #[test]
    fn test_gen_poly() {
        let gen = gen_polynomial(7);
        let gen_coef: Vec<u8> = gen.terms().map(|t| t.coef()).collect();
        assert_eq!(&gen_coef, &[1, 127, 122, 154, 164, 11, 68, 117]);

        let gen = gen_polynomial(10);
        let gen_coef: Vec<u8> = gen.terms().map(|t| t.coef()).collect();
        assert_eq!(&gen_coef, &[1, 216, 194, 159, 111, 199, 94, 95, 113, 157, 193]);

         let gen = gen_polynomial(15);
        let gen_coef: Vec<u8> = gen.terms().map(|t| t.coef()).collect();
        assert_eq!(&gen_coef, &[1, 29, 196, 111, 163, 112, 74, 10, 105, 105, 139, 132, 151, 32, 134, 26]);


    }

    #[test]
    fn test_poly_remainder() {
        // let poly_1 = Poly::from(&[0x12, 0x34, 0x56, 0x00, 0x00, 0x00, 0x00]);
        let poly_1 = Poly::from(6, &[18, 52,86, 0, 0, 0, 0]);
        // let divisor = Poly::from(&[0x01, 0x0f, 0x36, 0x78, 0x40]);
        let divisor = Poly::from(4,&[1, 15, 54, 120, 64]);
        let remainder = poly_1.div_remainder(&divisor);

        let expected_div = [18u8, 218, 223];
        let expected_rem = [55u8, 230, 120, 217];
        let remainder_terms: Vec<Term> = remainder.terms().collect();
        assert_eq!(&remainder_terms, &[Term(3, 55), Term(2,230),
                                        Term(1, 120), Term(0, 217)]);

    }

    #[test]
    fn test_remainder_from_code_words() {
        //[65, 22, 151, 54, 22, 150, 22, 130, 215, 6, 87, 39, 86, 214, 22, 198, 198, 19, 16]
        let data = Poly::from(18, &[0x41, 0x16, 0x97, 0x36, 0x16, 0x96, 0x16,
            0x82, 0xD7, 0x06, 0x57, 0x27, 0x56, 0xD6, 0x16, 0xC6, 0xC6, 0x13, 0x10]);
        let gen_poly = Poly::from(7, &[1, 127, 122, 154, 164, 11, 68, 117]);

        //multiply by x^7, same as degree of gen poly
        let data = data.multiply(Term(gen_poly.degree,1));

        let remainder = data.div_remainder(&gen_poly);
        let terms:Vec<Term> = remainder.terms().collect();

        assert_eq!(&terms, &[Term(6, 204), Term(5,74), Term(4,69),  Term(3,191),
                             Term(2,184), Term(1,184), Term(0,170)]);

//version 1 , Error level L
        let bytes = hex_str_to_bytes("40 86 97 36 16 96 16 82 D7 00 EC 11 EC 11 EC 11 EC 11 EC");
        assert_eq!((16*13 + 7), hex_byte("D7") , "0xD7 == 215");
        assert_eq!(&bytes, &[0x40, 0x86, 0x97, 0x36, 0x16, 0x96, 0x16, 0x82, 0xD7, 0x00, 0xEC, 0x11, 0xEC, 0x11, 0xEC, 0x11, 0xEC, 0x11, 0xEC]);
        let p = Poly::from(18, &bytes);
        let gen_poly = Poly::from(7, &[1, 127, 122, 154, 164, 11, 68, 117]);
        let data = p.multiply(Term(gen_poly.degree,1));
        let remainder = data.div_remainder(&gen_poly);
        let terms:Vec<u8> = remainder.terms().map(|t| t.coef()).collect();
        let expected:Vec<u8> = "5C 5A 9A 55 CB 35 7F".split_ascii_whitespace()
                                                .map(|s|hex_byte(s)).collect();
        assert_eq!(&terms, &expected);


    }


    #[test]
    fn test_remainder_from_long_code_words() {
        let data = hex_str_to_bytes("42 55 47 27 57 37 42 06 96 E2 07 46 86 52 04 C4 F5 24 42 07 76 97 46 82 06 16 C6 C2 07 96 F7 57 22 06 86 56 17 27 40 EC 11 EC 11 EC 11 EC 11 EC 11 EC 11 EC 11 EC 11");
        assert_eq!(data.len(), 55);
        let msg_poly = Poly::from((data.len() - 1) as u8, &data);
        let gen_poly = gen_polynomial(15);

        assert_eq!(15, gen_poly.degree);
        let msg_poly = msg_poly.multiply(Term(gen_poly.degree, 1));
        assert_eq!(msg_poly.degree, 54 + 15, "msg poly degree should be 69" );
        let expected_rem = hex_str_to_bytes("0D DD 7F 26 BA B3 13 5E D9 E4 66 D8 74 58 00");
        let rem = msg_poly.div_remainder(&gen_poly);
        let rem_terms: Vec<u8> = rem.terms().map(|t| {t.coef()}).collect();

        assert_eq!(rem_terms.len(), expected_rem.len(), "remainder poly length");
        assert_eq!(&rem_terms, &expected_rem);
    }

    #[test]
    fn test_remainder_from_long_code_words_version5() {
        let data = hex_str_to_bytes("45 15 47 27 57 37 42 06 96 E2 07 46 86 52 04 C4 F5 24 42 07 76 97 46 82 06 16 C6 C2 07 96 F7 52 06 86 56 17 27 42 06 C6 56 16 E2 06 E6 F7 42 06 F6 E2 07 96 F7 57 22 06 F7 76 E2 07 56 E6 46 57 27 37 46 16 E6 46 96 E6 72 05 07 26 F7 66 57 26 27 32 00 EC 11 EC 11 EC 11 EC 11 EC 11 EC 11 EC 11 EC 11 EC 11 EC 11 EC 11 EC 11 EC");
        println!("{:?}", &data);
        assert_eq!(data.len(), 108);
        let msg_poly = Poly::from((data.len() - 1) as u8, &data);
        let gen_poly = gen_polynomial(26);

        assert_eq!(26, gen_poly.degree);
        let msg_poly = msg_poly.multiply(Term(gen_poly.degree, 1));
        assert_eq!(msg_poly.degree, 107 + 26, "msg poly degree should be 69" );
        let expected_rem = hex_str_to_bytes("5E 9C 9F 79 9E B0 51 CC C1 27 EB D4 04 05 19 D9 BB 61 E8 93 76 3E 86 85 08 47");
        let rem = msg_poly.div_remainder(&gen_poly);
        let rem_terms: Vec<u8> = rem.terms().map(|t| {t.coef()}).collect();

        assert_eq!(rem_terms.len(), expected_rem.len(), "remainder poly length");
        assert_eq!(&rem_terms, &expected_rem);
    }

    #[test]
    fn test_remainder_from_long_code_words_0() {
        let data = hex_str_to_bytes("41 65 47 27 57 37 42 06 96 E2 07 46 86 52 04 C4 F5 24 42 07 76 97 46 80 EC 11 EC 11 EC 11 EC 11 EC 11");
        println!("{:?}", &data);
        assert_eq!(data.len(), 34);
        let msg_poly = Poly::from((data.len() - 1) as u8, &data);
        let gen_poly = Poly::from(10,
                                  &[1, 216, 194, 159, 111, 199, 94, 95, 113, 157, 193]);


        assert_eq!(10, gen_poly.degree);
        let msg_poly = msg_poly.multiply(Term(gen_poly.degree, 1));
        assert_eq!(msg_poly.degree, 33 + gen_poly.degree, "msg poly degree should be 44" );
        let expected_rem = hex_str_to_bytes("59 C5 2F 07 84 C7 CA DC 74 9A");
        let rem = msg_poly.div_remainder(&gen_poly);
        let rem_terms: Vec<u8> = rem.terms().map(|t| {t.coef()}).collect();
        println!("{:?}", &rem_terms);
        println!("{:?}", &expected_rem);
        assert_eq!(&rem_terms, &expected_rem);

    }

}
