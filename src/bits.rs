//iterate over bits in byte, starting with msb bit
pub struct MsbBitIter<'a> {
    bytes: &'a [u8],
    index: usize,
    bit_index: u8,
}

impl<'a> MsbBitIter<'a> {
    pub fn new(bytes: &[u8]) -> MsbBitIter {
        return MsbBitIter {
            bytes,
            index: 0,
            bit_index: 8, //start at most significant bit first
        };
    }
}

impl<'a> Iterator for MsbBitIter<'a> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bit_index == 0 {
            self.index += 1;
            self.bit_index = 8;
        }
        debug_assert!(self.bit_index > 0);
        if self.index < self.bytes.len() {
            let bit_i = self.bit_index - 1;
            let byte = self.bytes[self.index];
            let bit = byte & (1 << bit_i);
            self.bit_index -= 1;
            return Some(bit != 0);
        }
        None
    }
}

//writes bits to mut slice in big endian bit order
//eg append_bits(0b0010, 4) -> first bytes[0] -> 0b00100000 (32)
pub struct BigEndianBitWriter<'a> {
    bytes: &'a mut [u8],
    current_bit: u32,
}

impl<'a> BigEndianBitWriter<'a> {
    pub fn new(buffer: &mut [u8]) -> BigEndianBitWriter {
        BigEndianBitWriter {
            bytes: buffer,
            current_bit: 0,
        }
    }

    pub fn append_bits(&mut self, data: u8, num_of_bits: u8) {
        let mut slot = (self.current_bit / 8) as usize;
        let mut bit_index = self.current_bit % 8;
        for i in (0..num_of_bits).rev() {
            let is_bit_set = (data & (1 << i)) != 0;
            let mask = 1 << (7 - bit_index);
            if is_bit_set {
                self.bytes[slot] |= mask;
            } else {
                self.bytes[slot] &= !mask;
            }

            if bit_index == 7 {
                slot += 1;
                bit_index = 0;
            } else {
                bit_index += 1;
            }
        }
        self.current_bit = ((slot * 8) + bit_index as usize) as u32;
    }

    pub fn bits_written(&self) -> usize {
        self.current_bit as usize
    }
}

#[cfg(test)]
mod tests {
    use crate::bits::{BigEndianBitWriter, MsbBitIter};

    #[test]
    fn test_bit_msb_iter() {
        let data = [
            0x20, 0x21, 0xCD, 0x45, 0x20, 0xEC, 0x11, 0xEC, 0x11, 0xEC, 0x11, 0xEC, 0x11, 0xEC,
            0x11, 0xEC, 0x11, 0xEC, 0x11, 0x31, 0xB9, 0x38, 0x57, 0x4C, 0x1D, 0xE2,
        ];

        let expected_msb_bits_str = "0010000000100001110011010100010100100000111011000001000111101100000100011110110000010001111011000001000111101100000100011110110000010001111011000001000100110001101110010011100001010111010011000001110111100010";
        let bits: Vec<bool> = expected_msb_bits_str.chars().map(|c| c == '1').collect();
        let it = MsbBitIter::new(&data);
        for (i, (expected_bit, actual_bit)) in bits.iter().zip(it).enumerate() {
            assert_eq!(*expected_bit, actual_bit, "bit differ at position {}", i);
        }
    }

    #[test]
    fn test_bit_writer() {
        let mut bit_buff = [0; 8];
        let mut bit_writer = BigEndianBitWriter::new(&mut bit_buff);
        bit_writer.append_bits(0b0100, 4);

        assert_eq!(bit_writer.bits_written(), 4);
        bit_writer.append_bits(0b00000110, 8);
        bit_writer.append_bits(0b0110, 4);
        assert_eq!(bit_writer.bits_written(), 16);
        let mut actual = Vec::new();
        for i in 0..2 {
            let byte = bit_buff[i];
            for j in (0..8).rev() {
                let val = if (byte & (1 << j)) == 0 { '0' } else { '1' };
                actual.push(val);
            }
        }
        let expected: Vec<char> = "0100000001100110".chars().collect();
        assert_eq!(&expected, &actual)
    }
}
