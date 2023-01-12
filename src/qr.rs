use std::collections::HashSet;
use crate::bits::{BitSquare, MsbBitIter, Square};
use crate::error_cc::ErrorLevel;
use crate::qr::encode::{add_padding, encode_byte_segment};


pub fn version_to_size(v: u8) -> u8 {
    return 4*v + 17;
}

static  MASK_FN: [fn((u8,u8)) -> bool;2]  = [
                                            |(x, y)| { 0 == (x + y) % 2 },
                                            |(x, y)| { 0 == (x + y) % 2 } ];

pub struct QrCode {
    data: BitSquare,
    reserved_bits: BitSquare,
    version: u8,
    error_level: ErrorLevel
}



impl QrCode {

    pub fn new(version: u8, error_level: ErrorLevel) -> QrCode {
        let size = version_to_size(version);
        let mut bit_sq = BitSquare::new(size);
        let mut reserved = BitSquare::new(size);
        draw_timing_pattern(&mut bit_sq, &mut reserved);
        draw_finding_pattern(&mut bit_sq, &mut reserved);
        set_alignment_patterns(&mut bit_sq, version, &mut reserved);



        let mut qr = QrCode {
            data: bit_sq,
            reserved_bits: reserved,
            version,
            error_level,
        };

        qr.set_format();
        qr.set_dark_module();
        return qr;

    }

    pub fn encode_data(&mut self, data: &str) {
        let mut out_bytes = [0u8;256];
        let size = encode_byte_segment(data,
                                       &mut out_bytes).unwrap();
        let version = self.version;
        let code_words_size = ErrorLevel::L.data_code_words(version);
        let padding = code_words_size - size;
        add_padding(&mut out_bytes[size .. (size + padding)]);
        let size = ErrorLevel::L.add_error_codes(version, &mut out_bytes);
        self.set_code_words(&out_bytes[0..size]);
        self.apply_mask(0);
    }
    pub fn data_sq(&self) -> &BitSquare {
        return &self.data;
    }

    fn apply_mask(&mut self, mask_pattern: u8) {
        debug_assert!(mask_pattern < 8, "invalid mask patter {}", mask_pattern);
        let mask_fn = MASK_FN[mask_pattern as usize];
        let size = self.data.size;
        for x in 0..size {
            for y in 0..size {
                if self.is_data_module((x,y)) && mask_fn((x,y)) {
                    self.data.flip_bit(x,y);
                }
            }
        }
    }

    pub fn reserved_area(&self) -> &BitSquare {
        return &self.reserved_bits;
    }

    fn is_data_module(&self, pos: (u8,u8)) -> bool {
        let (x,y) = pos;
        return !self.reserved_bits.is_set(x, y);
    }



    fn set_format(&mut self) {
        let bits = self.error_level.format_bits(0);
        debug_assert!((bits >> 15) == 0, "format must be 15 bits");
        let bit = |i| 0 != (bits & (1u32 << i)); //is ith bit set
        for i in 0..6 {
            self.set_function_module(8, i, bit(i));
        }
        self.set_function_module(8,7, bit(6));
        self.set_function_module(8,8, bit(7));
        self.set_function_module(7,8, bit(8));

        //top horizontal part
        for i in 9 .. 15 {
            self.set_function_module(14 -i, 8, bit(i));
        }

        //second copy of version info
        //top right part
        for i in 0 .. 8 {
            self.set_function_module(self.data.size -1 - i, 8, bit(i));
        }
        //bottom left vertical
        for i in 8 .. 15 {
            self.set_function_module(8, self.data.size -15 + i, bit(i));
        }

    }

    fn set_function_module(&mut self, x:u8,y:u8, is_set: bool) {

        self.data.set_value(x, y, is_set);

        self.reserved_bits.set_value(x, y, true);
    }

    fn set_dark_module(&mut self) {
        let (x,y) = (8, 4*self.version + 9);
        self.set_function_module(x,y, true);
    }

    pub (crate) fn set_code_words(&mut self, code_words: &[u8]) {
        debug_assert!(code_words.len() == self.expected_byte_count(),
                      "code_words len for version {}, {} but was {}",
                      self.version, self.expected_byte_count(), code_words.len());

        let mut bit_iter = MsbBitIter::new(code_words);
        let zigzag_it = ZigzagIter::new(self.data.size);
        let mut bit_count:usize = 0;
        for (x,y) in zigzag_it {
            if !self.is_data_module((x, y)) {
                continue;
            }
            if let Some(bit) = bit_iter.next() {
                self.data.set_value(x,y, bit);
                bit_count += 1;
            }
            else {
                break;
            }
        }
        assert_eq!(bit_count, code_words.len()*8);
    }

    fn expected_byte_count(&self) -> usize {
        self.error_level.total_words(self.version)
    }
}


enum Step {
    Left,
    Up,
    Down
}


pub struct ZigzagIter {
    next_position: Option<(u8, u8)>,
    size: u8,
    traverse_up: bool //direction

}


impl ZigzagIter  {
    fn new(size:u8) -> ZigzagIter {

        return ZigzagIter {
            next_position: Some((size - 1, size - 1)), //bottom right corner
            size,
            traverse_up: true
        }
    }



    fn next_step(&self) -> Step {
        let (x, _) = self.next_position.unwrap();
        let start_x = self.size - 1;
        //if started in odd x then every odd x would move left
        if (start_x & 1) == (x & 1) {
            return Step::Left;
        }
        return if self.traverse_up { Step::Up } else { Step::Down }
    }

    fn end_position(&self) -> (u8, u8) {
        let start_x = self.size -1;
        return if (start_x & 1) != 0 {
            (0, self.size -1)
        } else {
            (0, 0)
        }
    }

    fn next_pos(&mut self) -> Option<(u8,u8)> {
        if self.next_position.is_none() {
            return None;
        }

        let (x, y) = self.next_position.unwrap();
        if (x,y) == self.end_position() { //currently in end position
            self.next_position = None;
            return Some((x,y));
        }
        let size = self.size;
        let next_step = self.next_step();
        match (x, y, next_step) {
            (0, y, Step::Left) => {
                self.next_position = Some((0, y -1));
            }
            (x, 0, Step::Up) => {
                self.next_position = Some((x -1, y));
                self.traverse_up = !self.traverse_up
            }
            (x, y, Step::Down) if (y + 1 == size) => {
                self.next_position = Some((x -1, y));
                self.traverse_up = !self.traverse_up
            }
            (x, y, Step::Left) => {
                self.next_position = Some((x - 1, y));
            }
            (x, y, Step::Up) => {
                self.next_position = Some((x +1, y -1));
            }
            (x, y, Step::Down) => {
                self.next_position = Some((x + 1, y + 1));
            }
        }
        return Some((x,y));

    }
}

impl Iterator for ZigzagIter {
    type Item = (u8, u8);

    fn next(&mut self) -> Option<Self::Item> {
        return self.next_pos();
    }
}



 fn set_alignment_patterns(sq: &mut BitSquare, version: u8, reserved: &mut BitSquare) {
    assert!(version <= 5, "not supported for higher versions");
    match version {
        2 => {
            alignment_square(sq, (18,18), reserved);
        }
        3 => {
            alignment_square(sq, (22,22), reserved);
        }
        _ => {}
    }
}


fn  alignment_square(sq: &mut BitSquare, top_left: (u8, u8), changes: &mut BitSquare) {
    let (x,y) = top_left;
    sq.set_square(Square::new(1, (x, y)), true);
    sq.set_square(Square::new(3, (x-1, y-1)), false);
    sq.set_square(Square::new(5, (x-2, y-2)), true);

    changes.set_square(Square::new(1, (x, y)), true);
    changes.set_square(Square::new(3, (x-1, y-1)), true);
    changes.set_square(Square::new(5, (x-2, y-2)), true);
}

    fn draw_timing_pattern(sq: &mut BitSquare, changes: &mut BitSquare) {
    let size = sq.size;
    for y in 0 .. size {
        let even = (y & 1) == 0;
        sq.set_value(6, y, even);
        changes.set_value(6, y, true);
    }

    for x in 0 .. size {
        let even = (x & 1) == 0;
        sq.set_value(x, 6, even);
        changes.set_value(x, 6, true);
    }
}

 fn draw_finding_pattern(sq: &mut BitSquare, changes: &mut BitSquare) {
    let size = sq.size;
    let is_dark = false;
    //top left
    finding_pattern(sq, (0, 0), changes);

    //separators
    sq.draw_vert((7,0), 8, is_dark);
    changes.draw_vert((7,0), 8, true);

     sq.draw_horizontal((0,7), 8, is_dark);
     changes.draw_horizontal((0,7), 8, true);


    //top right
    finding_pattern(sq, (size - 7, 0), changes);

    //separators
    sq.draw_vert((size - 8, 0), 8, is_dark);
    changes.draw_vert((size - 8, 0), 8, true);
    sq.draw_horizontal((size - 8 , 7), 8, is_dark);
    changes.draw_horizontal((size - 8 , 7), 8, true);

    //bottom right
    finding_pattern(sq, (0, size - 7), changes);

    //separators
    sq.draw_horizontal((0, size - 8), 8, is_dark);
    changes.draw_horizontal((0, size - 8), 8, true);
    sq.draw_vert((7 , size - 8), 8, is_dark);
    changes.draw_vert((7 , size - 8), 8, true);

}

fn finding_pattern(sq: &mut BitSquare, top_left: (u8, u8), changes: &mut BitSquare) {
    sq.set_square(Square::new(7, top_left), true);

    let (x,y) = top_left;
    sq.set_square(Square::new(5, (x + 1, y + 1)), false);
    sq.set_square(Square::new(3, (x + 2, y + 2)), true);
    sq.set_square(Square::new(1, (x + 3, y + 3)), true);

    changes.set_square(Square::new(7, top_left), true);
    changes.set_square(Square::new(5, (x + 1, y + 1)), true);
    changes.set_square(Square::new(3, (x + 2, y + 2)), true);
    changes.set_square(Square::new(1, (x + 3, y + 3)), true);
}


pub mod encode {
    use crate::bits::BigEndianBitWriter;


    #[derive(Debug)]
    pub enum EnumEncodingErr {
        NotAscii,
        NotAlphaNumeric,
        DataTooLong
    }

    // encode string to data code words
//include mode type and padding bits
    pub fn encode_byte_segment(data: &str, out: &mut [u8]) -> Result<usize, EnumEncodingErr> {
        let mut bit_writer = BigEndianBitWriter::new(out);
        let char_count = data.chars().count();

        let segment_mode = 0b0100; //bytes
        bit_writer.append_bits(segment_mode, 4);
        bit_writer.append_bits(char_count as u8, 8);
        for ch in data.chars() {
            let byte = ch as u8;
            bit_writer.append_bits(byte, 8);
        }
        bit_writer.append_bits(0b0000, 4); // terminator bits

        let pad_bits = bit_writer.bits_written() % 8;
        bit_writer.append_bits(0, pad_bits as u8);

        let bytes = bit_writer.bits_written() >> 3; //bits/8
        debug_assert!(bytes == 2 + char_count);
        return Ok(bytes);
    }



    pub fn add_padding(bytes: &mut [u8]) {
        let pad_bytes = [0xEC, 0x11];
        for i in 0..bytes.len() {
            let b = pad_bytes[(i & 1)]; //cycle between odd and even
            bytes[i] = b;
        }
    }
}


#[test]
pub fn test_encode_to_bytes() {
    let mut out_bytess = [0u8; 64];
    let res = encode_byte_segment("isaiah", &mut out_bytess);
    assert_eq!(res.unwrap(), 8);
    let expected_bytes = [0x40, 0x66, 0x97, 0x36, 0x16, 0x96, 0x16, 0x80];
    assert_eq!(&expected_bytes, &out_bytess[0..expected_bytes.len()]);

    let res = encode_byte_segment("isaiah-perumalla", &mut out_bytess);
    assert_eq!(res.unwrap(), 18);
    let expected_bytes = [0x41, 0x06, 0x97, 0x36, 0x16, 0x96, 0x16, 0x82, 0xD7,
        0x06, 0x57, 0x27, 0x56, 0xD6, 0x16, 0xC6, 0xC6, 0x10];
    assert_eq!(&expected_bytes, &out_bytess[0..expected_bytes.len()]);
}


#[test]
fn test_zigzag_iter() {
    let qr_4 = ZigzagIter::new(4);
    let steps:Vec<(u8,u8)> = qr_4.collect();

    assert_eq!(4*4, steps.len());
    assert_eq!(steps, vec![(3,3),(2,3), (3, 2), (2, 2),
                           (3, 1), (2, 1), (3, 0), (2, 0),
                           (1, 0), (0, 0), (1, 1), (0, 1),
                           (1, 2), (0, 2), (1, 3), (0, 3)]);

    let qr_5 = ZigzagIter::new(5);
    let steps_5:Vec<(u8,u8)> = qr_5.collect();
    assert_eq!(5*5, steps_5.len());
    assert_eq!(steps_5, vec![(4, 4), (3, 4), (4, 3), (3, 3), (4, 2), (3, 2), (4, 1), (3, 1), (4, 0),
                             (3, 0), (2, 0), (1, 0), (2, 1), (1, 1), (2, 2), (1, 2), (2, 3), (1, 3),
                             (2, 4), (1, 4), (0, 4), (0, 3), (0, 2), (0, 1), (0, 0)]);

    let qr_21 = ZigzagIter::new(21);
    let steps:Vec<(u8,u8)> = qr_21.collect();
    assert_eq!(21*21, steps.len());

}

#[test]
fn test_qr_data_module_iter() {
    let qr = QrCode::new(1, ErrorLevel::L);
    let data_modules:Vec<(u8,u8)> = ZigzagIter::new(qr.data.size).filter(|p| qr.is_data_module(*p)).collect();
    let data_modules_set:HashSet<(u8,u8)> = data_modules.iter().copied().collect();
    assert_eq!(data_modules_set.len(), data_modules.len());
    for i in 0..8 {
        let pos = (7, i);
        assert_eq!(false, data_modules_set.contains(&pos), "separator (7,{}) should not be in data modules",i);
    }
    let top_left_square = Square::new(9, (0,0)); //includes separator and format area

    let top_right_square = Square::new(9, (13, 0)); //includes separator and format area
    let bottom_left_square = Square::new(9, (0, 13)); //includes separator and format area

    assert_eq!(true, qr.data.is_set(8, 13), "dark module not set ({},{})",  8, 13);
    //square above dark module
    assert_eq!(true, qr.is_data_module((8, 12) ), "should be data module ({},{})", 8,12);
    assert_eq!(true, qr.is_data_module((7, 12) ), "should be data module ({},{})", 7,12);
    assert_eq!(false, qr.is_data_module((6, 12) ), "should NOT be data module ({},{})", 6,12);
    for i in 0..6 {
        assert_eq!(true, qr.is_data_module((i, 12) ), "should be data module ({},{})", i,12);
    }
    for point in &data_modules {

        assert_eq!(false, top_left_square.contains_point(*point), "{:?} is not a data module", *point);
        assert_eq!(false, top_right_square.contains_point(*point), "{:?} is not a data module", *point);
        assert_eq!(false, bottom_left_square.contains_point(*point), "{:?} is not a data module", *point);

        assert_eq!(false, qr.data.is_set(point.0, point.1), "{:?} data should be clear ", *point);
    }
    for i in 0..8 {
        let pos = (7, i);
        assert_eq!(false, data_modules_set.contains(&pos), "separator (7,{}) should not be in data modules",i);
    }

    println!("len={},{:?}", data_modules.len(), data_modules);
}


#[test]
fn test_basic_qr() {
    let mut qr = QrCode::new(1, ErrorLevel::L);
    qr.encode_data("isaiah-perumalla1");
    //remove mask
    qr.apply_mask(0);
    let mut bit_string = String::new();
    for (x,y) in ZigzagIter::new(qr.data.size).filter(|p| qr.is_data_module(*p)) {
        let bit = qr.data.is_set(x,y);
        if bit {
            bit_string.push('1');
        }
        else {
            bit_string.push('0');
        }
    }
    let expected_unmasked_str = "0100000100010110100101110011011000010110100101100001011010000010110101110000011001010111001001110101011011010110000101101100011011000110000100110001000011001100010010100100010110111111101110001011100010101010";
    assert_eq!(expected_unmasked_str.len(), bit_string.len());
    assert_eq!(expected_unmasked_str, bit_string);
}