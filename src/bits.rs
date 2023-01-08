

pub struct BitSquare {
    pub(crate) size: u8,
    bit_vec: Vec<u8>
}

impl BitSquare {
    pub fn new(size: u8) -> BitSquare {
        let n:u32 = (size as usize * size as usize) as u32;
        let bytes = (8 * ((n/ 8) + 1))/8;
        return BitSquare {
            size,
            bit_vec: vec![0; bytes as usize]
        }
    }

    pub fn is_set(&self, x: u8, y: u8) -> bool {
        assert!(x < self.size && y < self.size);
        let index = self.bit_index(x, y);
        let slot = index/8;
        let mask = 1 << (index % 8);
        return (self.bit_vec[slot as usize] & mask) != 0;
    }

    fn bit_index(&self, x: u8, y: u8) -> usize {
        let index = y as usize * (self.size as usize) + x as usize;
        index
    }

    //set/clear bits outlines by square sq
    pub fn set_square(&mut self, sq: Square, is_set: bool) {

        let top_left = sq.top_left;
        let len = sq.size;
        self.draw_horizontal(top_left, len, is_set);
        self.draw_vert(top_left, len, is_set);
        let bottom_left = (top_left.0, top_left.1 + len -1);
        self.draw_horizontal(bottom_left, len, is_set);
        let top_right = (top_left.0 + len -1, top_left.1);
        self.draw_vert(top_right, len, is_set);
    }

    pub fn flip_bit(&mut self, x:u8, y:u8) {
        assert!(x < self.size && y < self.size);
        let val = self.is_set(x, y);
        self.set_value(x,y, !val);
    }

    pub fn set_value(&mut self, x:u8, y:u8, val:bool) {
        assert!(x < self.size && y < self.size);
        let index = self.bit_index(x, y);
        let slot = index/8;
        let mask = 1 << (index % 8);
        if val {
            self.bit_vec[slot as usize] |= mask;
        }
        else {
            self.bit_vec[slot as usize] &= !mask;
        }

    }

    pub fn draw_vert(&mut self, from: (u8, u8), len: u8, is_set: bool) {
        for i in 0 .. len {
            self.set_value(from.0, from.1 + i, is_set);

        }
    }

    pub fn draw_horizontal(&mut self, from: (u8, u8), len: u8, is_set: bool) {
        for i in 0 .. len {
            self.set_value(from.0 + i, from.1, is_set);
        }
    }


}




pub struct Square {
    top_left: (u8,u8),
    size: u8
}

impl Square {
    pub fn contains_point(&self, point:(u8,u8)) -> bool {
        let (x,y) = point;
        let (top_left_x, top_left_y) = self.top_left;
        let size = self.size;
        return x >= top_left_x &&  x < (top_left_x + size) &&
                y >= top_left_y && y < (top_left_y + size);
    }
}

impl Square {

    pub fn new(size: u8, top_left: (u8, u8)) -> Square {
        return Square {
            top_left, size
        }
    }

    pub fn increment(&self) -> Square {
        assert_ne!(self.top_left.0, 0, "top_left x cannot be zero");
        assert_ne!(self.top_left.1, 0, "top_left y cannot be zero");
        return Square {
            top_left: (self.top_left.0 -1, self.top_left.1 -1),
            size: self.size + 1
        }
    }
}


#[test]
fn test_bit_square() {
    let square_size = 50;
    let mut bit_sq = BitSquare::new(square_size);
    for x in 0 ..square_size {
        for y in 0..square_size {
            assert_eq!((&bit_sq).is_set(x, y), false);
            (&mut bit_sq).set_value(x, y, true);
            assert_eq!((&bit_sq).is_set(x, y), true);
            (&mut bit_sq).set_value(x, y, false);
            assert_eq!((&bit_sq).is_set(x, y), false);
        }
    }
}

fn bit_msb_iter(bytes: &[u8]) -> BitIter {
    return BitIter::new(bytes);
}

pub struct BitIter<'a> {
    bytes: &'a [u8],
    index: usize,
    bit_index: u8
}

impl<'a> BitIter<'a> {
    pub fn new(bytes: &[u8]) -> BitIter {
        return BitIter {
            bytes,
            index: 0,
            bit_index: 8 //start at most significant bit first
        }
    }
}

impl<'a> Iterator for BitIter<'a> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bit_index == 0 {
            self.index += 1;
            self.bit_index = 8;
        }
        debug_assert!(self.bit_index > 0);
        if self.index < self.bytes.len() {
            let bit_i = self.bit_index -1;
            let byte = self.bytes[self.index];
            let bit = byte & (1 << bit_i);
            self.bit_index -= 1;
            return Some(bit != 0);
        }
        None

    }
}

#[test]
fn test_square_contains() {
    let sq = Square::new(9, (0,0));

    assert_eq!(true, sq.contains_point((0,0)));
    assert_eq!(true, sq.contains_point((0,8)));
    assert_eq!(true, sq.contains_point((1,1)));
    assert_eq!(true, sq.contains_point((8,8)));
    assert_eq!(false, sq.contains_point((9,0)));
}

#[test]
fn test_bit_msb_iter() {
    let data = [0x20, 0x21, 0xCD, 0x45, 0x20, 0xEC, 0x11, 0xEC, 0x11, 0xEC, 0x11, 0xEC, 0x11,
                        0xEC, 0x11, 0xEC, 0x11, 0xEC, 0x11, 0x31, 0xB9, 0x38, 0x57, 0x4C, 0x1D, 0xE2];

    let expected_msb_bits_str = "0010000000100001110011010100010100100000111011000001000111101100000100011110110000010001111011000001000111101100000100011110110000010001111011000001000100110001101110010011100001010111010011000001110111100010";
    let bits: Vec<bool> = expected_msb_bits_str.chars().map(|c| c == '1').collect();
    let it = bit_msb_iter(&data);
    for (i, (expected_bit, actual_bit)) in bits.iter().zip(it).enumerate() {
        assert_eq!(*expected_bit, actual_bit, "bit differ at position {}", i);
    }
}

