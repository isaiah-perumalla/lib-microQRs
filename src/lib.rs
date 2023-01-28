extern crate core;

use crate::bits::{BigEndianBitWriter, MsbBitIter};
use EncodingErr::DataTooLong;
use crate::codec::{MASK_FN, MaskFN};
use crate::error_cc::ErrorLevel;
use std::fs::File;
use std::io::{Read, Write};


pub mod bits;
pub mod codec;
pub mod error_cc;
pub mod gf256;

pub fn encode<const S: usize>(data: &str) -> Result<Code<S>, EncodingErr> {
    let mut encoded = [0; S];
    const MAX_VERSION: u8 = 5u8;
    let size = encode_byte_segment(data, &mut encoded)?;
    if size > S {
        return Err(DataTooLong);
    }
    let err_level = ErrorLevel::L;
    let v = (1..=MAX_VERSION)
        .filter(|v| err_level.data_code_words(*v) >= size)
        .map(|v| Version(v))
        .next();
    if v.is_none() {
        return Err(DataTooLong);
    }
    let version = v.unwrap();
    if err_level.total_words(version.0) >= S {
        return Err(DataTooLong);
    }
    let padding = err_level.data_code_words(version.0) - size;
    add_padding(&mut encoded[size..(size + padding)]);
    let size = err_level.add_error_codes(version.0, &mut encoded);
    let code_words = &encoded[0..size];
    let expected_bytes = err_level.total_words(version.0);

    debug_assert!(
        code_words.len() == expected_bytes,
        "code_words len for version {}, {} but was {}",
        version.0,
        expected_bytes,
        code_words.len()
    );
    Ok(Code {
        version,
        err_level,
        data: encoded,
    })
}

pub struct Code<const S: usize> {
    pub version: Version,
    pub err_level: ErrorLevel,
    pub data: [u8; S],
}

impl<const S: usize> Code<S> {
    pub fn code_words(&self) -> &[u8] {
        let num_words = self.err_level.total_words(self.version.0);
        &self.data[0..num_words]
    }

    pub fn data_module_iter(&self) -> impl Iterator<Item =((u8,u8), bool)> + '_{
        let version_num = self.version.0;
        let num_words = self.err_level.total_words(version_num);
        let code_words = &self.data[0..num_words];
         let mut bit_iter = MsbBitIter::new(code_words);
        let mut data_it = Version(version_num).data_region_iter();
        std::iter::zip(data_it, bit_iter)

    }

    pub fn module_iter(&self) -> impl Iterator<Item = Module> + '_ {
        let mask_level = 0;
        let version_num = self.version.0;
        let format_modules = self.version.format_modules(self.err_level, mask_level);
        println!("{:?}", &format_modules);
        let mut reserved_it = Version(version_num).reserved_iter();
        let mut data_it = Version(version_num).data_region_iter();

        let num_words = self.err_level.total_words(version_num);
        let code_words = &self.data[0..num_words];
        let mut bit_iter = MsbBitIter::new(code_words);
        let mut format_index = 0;
        std::iter::from_fn(move || {
            if let Some(m) = reserved_it.next() {
                Some(m)
            } else if let Some((x, y)) = data_it.next() {
                let bit = match bit_iter.next() {
                    Some(bit) => bit,
                    _ => false,
                };
                if MASK_FN[mask_level as usize]((x, y)) {
                    Some(Module::data((x, y), !bit))
                } else {
                    Some(Module::data((x, y), bit))
                }
            } else if format_index < 30 {
                let i = format_index;
                format_index += 1;
                Some(format_modules[i])
            } else {
                None
            }
        })
    }
}
#[derive(Copy, Clone, Debug)]
pub struct Module((u8, u8), u8); //position and flags
impl Module {
    const IS_DARK_MASK: u8 = 1u8 << 7;
    const TYPE_MASK: u8 = 0xF;
    const TYPE_DATA: u8 = 0x1;
    const TYPE_RESERVED: u8 = 0x0;
    pub fn is_dark(&self) -> bool {
        let flags = self.1;
        0 != flags & Self::IS_DARK_MASK // bit 0 is is dark
    }

    pub fn position(&self) -> (u8, u8) {
        self.0
    }
    pub fn data(position: (u8, u8), is_dark: bool) -> Module {
        let mut flags = Self::TYPE_DATA;
        if is_dark {
            flags |= Self::IS_DARK_MASK;
        }
        Module(position, flags)
    }

    pub fn reserved(position: (u8, u8), is_dark: bool) -> Module {
        let mut flags = Self::TYPE_RESERVED;
        if is_dark {
            flags |= Self::IS_DARK_MASK;
        }
        Module(position, flags)
    }

    pub fn is_data(&self) -> bool {
        let flags = self.1;
        Self::TYPE_DATA == (flags & Self::TYPE_MASK)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Version(pub u8);

impl Version {
    //alignment square position for version 1-5
    const ALIGNMENT_POSITIONS: [&'static [(u8, u8)]; 6] =
        [&[], &[], &[(18, 18)], &[(22, 22)], &[(26, 26)], &[(30, 30)]];
    pub fn square_size(&self) -> u8 {
        4 * self.0 + 17
    }

    pub fn format_modules(&self, err_level: ErrorLevel, mask_level: u8) -> [Module; 30] {
        let mut mask_module = [Module::reserved((0, 0), false); 30];
        let new_mod = |pos, bit| Module::reserved(pos, bit);
        let mut index = 0;
        let bits = err_level.format_bits(mask_level);
        debug_assert!((bits >> 15) == 0, "format must be 15 bits");
        let bit = |i| 0 != (bits & (1u32 << i)); //is ith bit set
        for i in 0..6 {
            mask_module[index] = new_mod((8, i), bit(i));
            index += 1;
        }
        mask_module[index] = new_mod((8, 7), bit(6));
        index += 1;
        mask_module[index] = new_mod((8, 8), bit(7));
        index += 1;
        mask_module[index] = new_mod((7, 8), bit(8));
        index += 1;
        let square_size = self.square_size();
        //top horizontal part
        for i in 9..15 {
            mask_module[index] = new_mod((14 - i, 8), bit(i));
            index += 1;
        }

        //second copy of version info
        //top right part
        for i in 0..8 {
            mask_module[index] = new_mod((square_size - 1 - i, 8), bit(i));
            index += 1;
        }
        //bottom left vertical
        for i in 8..15 {
            mask_module[index] = new_mod((8, square_size - 15 + i), bit(i));
            index += 1;
        }
        mask_module
    }
    fn dark_module_pos(&self) -> (u8, u8) {
        (8, 4 * self.0 + 9)
    }

    fn timing_pattern_iter(&self) -> impl Iterator<Item = (u8, u8, bool)> {
        let size = self.square_size();
        let mut y = 8;
        let mut x = 8;

        std::iter::from_fn(move || {
            if y < size - 8 {
                let is_dark = (y & 1) == 0;
                let pos = y;
                y += 1;
                Some((6, pos, is_dark))
            } else if x < size - 8 {
                let is_dark = (x & 1) == 0;
                let pos = x;
                x += 1;
                Some((pos, 6, is_dark))
            } else {
                None
            }
        })
    }

    fn separator_squares_iter(&self) -> impl Iterator<Item = (u8, u8, bool)> {
        let horizonal = |x, y| (0u8..8).map(move |i| (x + i, y));
        let vert = |x, y| (0u8..8).map(move |i| (x, y + i));
        let size = self.square_size();
        let mut separators = horizonal(0, 7)
            .chain(vert(7, 0))
            .chain(vert(size - 8, 0))
            .chain(horizonal(size - 8, 7))
            .chain(horizonal(0, size - 8))
            .chain(vert(7, size - 8));
        const WHITE: bool = false;
        std::iter::from_fn(move || {
            if let Some((x, y)) = separators.next() {
                Some((x, y, WHITE))
            } else {
                None
            }
        })
    }
    fn dark_module_location(&self) -> (u8, u8) {
        (8, (4 * self.0 + 9))
    }

    pub fn is_data_location(&self, location: (u8, u8)) -> bool {
        let (x, y) = location;
        if x == 6 || y == 6 {
            return false; // timing area
        }
        if location == self.dark_module_location() {
            return false;
        }
        let size = self.square_size();
        let reserved = [
            Rect((0, 0), (8, 8)),               //top_left includes version info area
            Rect((size - 8, 0), (size - 1, 8)), //top_right
            Rect((0, size - 8), (8, size - 1)),
        ];

        if reserved.iter().any(|rect| rect.contains(location)) {
            return false;
        }
        if self
            .alignment_squares_iter()
            .any(|sq| sq.contains(location))
        {
            return false;
        }
        true
    }
    fn alignment_square(center: (u8, u8)) -> ConcentricSquare {
        ConcentricSquare {
            center,
            size: 3,
            color_bits: 0b101,
        }
    }

    fn alignment_squares_iter(&self) -> impl Iterator<Item = ConcentricSquare> {
        let mut i = 0;
        let square_positions = Self::ALIGNMENT_POSITIONS[self.0 as usize];
        std::iter::from_fn(move || {
            if i < square_positions.len() {
                let pos = square_positions[i];
                i += 1;
                Some(Self::alignment_square(pos))
            } else {
                None
            }
        })
    }

    fn finding_pattern(&self) -> impl Iterator<Item = ConcentricSquare> {
        let size = self.square_size();
        let mut i = 0;
        let squares = [
            ConcentricSquare {
                center: (3, 3),
                size: 4,
                color_bits: 0b1011,
            },
            ConcentricSquare {
                center: (size - 4, 3),
                size: 4,
                color_bits: 0b1011,
            },
            ConcentricSquare {
                center: (3, size - 4),
                size: 4,
                color_bits: 0b1011,
            },
        ];
        std::iter::from_fn(move || {
            if i < 3 {
                i += 1;
                Some(squares[i - 1])
            } else {
                None
            }
        })
    }

    pub fn data_region_iter(&self) -> impl Iterator<Item = (u8, u8)> {
        let size = self.square_size();
        let mut iter = ZigzagIter::new(size);
        let v = Version(self.0);
        iter.filter(move |pos| v.is_data_location(*pos))
    }

    pub fn reserved_iter(&self) -> impl Iterator<Item = Module> {
        let to_module: fn((u8, u8, bool)) -> Module =
            |(x, y, is_dark)| Module::reserved((x, y), is_dark);
        let mut finding_pattern_it = self
            .finding_pattern()
            .flat_map(move |sq| sq.iter_squares().map(to_module));
        let mut timing_iter = self.timing_pattern_iter().map(to_module);
        let mut alignment_square_iter = self
            .alignment_squares_iter()
            .flat_map(move |it| it.iter_squares().map(to_module));

        let dark_module = Module::reserved(self.dark_module_location(), true);
        let mut seperator_iter = self
            .separator_squares_iter()
            .map(|(x, y, is_dark)| Module::reserved((x, y), is_dark))
            .chain(std::iter::once(dark_module));
        std::iter::from_fn(move || {
            if let Some(v) = finding_pattern_it.next() {
                Some(v)
            } else if let Some(v) = alignment_square_iter.next() {
                Some(v)
            } else if let Some(v) = timing_iter.next() {
                Some(v)
            } else if let Some(v) = seperator_iter.next() {
                Some(v)
            } else {
                None
            }
        })
    }
}

//rectagle enclosed by top_right and bottom_right
#[derive(Copy, Clone, Debug)]
pub(crate) struct Rect((u8, u8), (u8, u8));

impl Rect {
    pub(crate) fn contains(&self, point: (u8, u8)) -> bool {
        let (top_left, bottom_right) = (self.0, self.1);
        let (x, y) = point;
        x >= top_left.0 && x <= bottom_right.0 && y >= top_left.1 && y <= bottom_right.1
    }
}

#[derive(Copy, Clone, Debug)]
struct ConcentricSquare {
    center: (u8, u8),
    size: u8,
    color_bits: u8,
}

impl ConcentricSquare {
    const EMPTY: ConcentricSquare = ConcentricSquare {
        center: (0, 0),
        size: 0,
        color_bits: 0,
    };

    fn contains(&self, location: (u8, u8)) -> bool {
        let size = self.size;
        let (top_left_x, top_left_y) = (self.center.0 + 1 - size, self.center.1 + 1 - size);
        let (bottom_right_x, bottom_right_y) = (self.center.0 + size - 1, self.center.1 + size - 1);
        let (x, y) = location;
        x >= top_left_x && x <= bottom_right_x && y >= top_left_y && y <= bottom_right_y
    }

    fn square_points(&self, out: &mut [(u8, u8, bool)]) -> usize {
        let mut count = 0;
        let is_dark = 0 != (self.color_bits & 1);
        out[0] = (self.center.0, self.center.1, is_dark);
        count += 1;

        let (center_x, center_y) = self.center;
        for i in 1..self.size {
            let (top_right_x, top_right_y) = (center_x - i, center_y - i);
            let delta = 2 * i + 1;
            let is_dark = 0 != (self.color_bits & (1 << i));
            for x in top_right_x..(top_right_x + delta) {
                out[count] = (x, top_right_y, is_dark);
                out[count + 1] = (x, (top_right_y + delta - 1), is_dark);
                count += 2;
            }
            for y in (top_right_y + 1)..(top_right_y + delta - 1) {
                out[count] = (top_right_x, y, is_dark);
                out[count + 1] = ((top_right_x + delta - 1), y, is_dark);
                count += 2;
            }
        }
        count
    }

    pub(crate) fn iter_squares(&self) -> impl Iterator<Item = (u8, u8, bool)> {
        let mut i = 0;
        let mut values = [(0, 0, false); 64];
        let count = self.square_points(&mut values);

        std::iter::from_fn(move || {
            if i < count {
                let item = values[i];
                i += 1;
                Some(item)
            } else {
                None
            }
        })
    }
}

pub const WHITE: RGB = RGB(255, 255, 255);
pub const RED: RGB = RGB(255, 0, 0);

pub const GREY: RGB = RGB(169, 169, 169);
pub const GREEN: RGB = RGB(0, 255, 0);
pub const YELLOW: RGB = RGB(255, 0, 255);
pub const ORANGE: RGB = RGB(255, 165, 0);

pub const BLACK: RGB = RGB(0, 0, 0);

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RGB(u8, u8, u8);

fn serialize_rgb(pixels: &Vec<RGB>, size: usize) -> Vec<u8> {
    let mut output: Vec<u8> = Vec::with_capacity(size * 3);
    for pix in pixels {
        output.push(pix.0);
        output.push(pix.1);
        output.push(pix.2);
    }
    output
}

pub struct Canvas {
    pixels: Vec<RGB>,
    width: u32,
    height: u32,
    pixel_size: u8,
    quite_zone: u8,
}

impl Canvas {
    const DEFAULT_QUITE_ZONE_SIZE: u8 = 2;
    const PIXEL_PER_MOD: u8 = 16;
    pub fn set_colour(&mut self, x: u32, y: u32, colour: &RGB) {
        // make this more natural? In C++ you can overload () to get a functor
        if x > 0 && y > 0 && x < self.width && y < self.height {
            self.pixels[(self.width * y + x) as usize] = *colour;
        }
    }

    pub fn write_to_file(&mut self, filename: &str) {
        let mut file = init_ppm(filename, self.width, self.height);
        let bytes = &serialize_rgb(&self.pixels, (self.width * self.height) as usize);
        file.write_all(bytes).expect("error");
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: &RGB) {
        let pixel_size = self.pixel_size as u32;
        let quite_zone = self.quite_zone as u32;
        for i in 0..pixel_size {
            for j in 0..pixel_size {
                self.set_colour(
                    (x + quite_zone) * pixel_size + i,
                    (y + quite_zone) * pixel_size + j,
                    &color,
                );
            }
        }
    }

    pub fn for_version(v: Version) -> Canvas {
        let size = v.square_size() as u32;
        let canvas_size: u32 =
            ((size + Self::DEFAULT_QUITE_ZONE_SIZE as u32 * 2) * Self::PIXEL_PER_MOD as u32) as u32;

        Canvas::new(
            canvas_size,
            canvas_size,
            GREY,
            Self::DEFAULT_QUITE_ZONE_SIZE,
            Self::PIXEL_PER_MOD,
        )
    }

    pub fn new(width: u32, height: u32, bg_color: RGB, quite_zone: u8, pixel_size: u8) -> Canvas {
        Canvas {
            width,
            height,
            quite_zone: quite_zone,
            pixel_size: pixel_size,
            pixels: vec![bg_color; (width * height) as usize],
        }
    }
}

fn init_ppm(filename: &str, width: u32, height: u32) -> File {
    let mut file = File::create(format!("{}.ppm", filename)).expect("couldn't create");
    file.write_all(format!("P6 {} {} 255 ", width, height).as_bytes())
        .expect("error writing to a file");
    file
}
#[cfg(test)]
mod tests;


#[derive(Debug)]
pub enum EncodingErr {
    NotAscii,
    NotAlphaNumeric,
    DataTooLong,
}


pub(crate) struct ZigzagIter {
    next_position: Option<(u8, u8)>,
    size: u8,
    traverse_up: bool, //direction
}

impl ZigzagIter {
    pub(crate) fn new(size: u8) -> ZigzagIter {
        return ZigzagIter {
            next_position: Some((size - 1, size - 1)), //bottom right corner
            size,
            traverse_up: true,
        };
    }
}

impl Iterator for ZigzagIter {
    type Item = (u8, u8);

    fn next(&mut self) -> Option<Self::Item> {
        const X_ODD: bool = true;
        const X_EVEN: bool = false;
        const UP: bool = true;
        const DOWN: bool = false;
        let size = self.size;
        //key observation is when x is even  next move is left
        if let Some((x, y)) = self.next_position {
            let x_odd = (x & 1) != 0;
            let next_pos = match (x, y, x_odd, self.traverse_up) {
                (6, _, _, _) => Some((5, 0)), //column with timing pattern line
                (0..=5, y, X_ODD, _) | (7.., y, X_EVEN, _) => Some((x - 1, y)), //always left
                (0..=5, 0, X_EVEN, UP) if x > 0 => {
                    self.traverse_up = false;
                    Some((x - 1, y))
                }
                (0..=5, y, X_EVEN, UP) if y > 0 => Some((x + 1, y - 1)),
                (0..=5, y, X_EVEN, DOWN) => {
                    if x == 0 && (y + 1) == size {
                        None
                    } else if y + 1 == size {
                        self.traverse_up = !self.traverse_up;
                        Some((x - 1, y))
                    } else {
                        Some((x + 1, y + 1))
                    }
                }
                (7.., 0, X_ODD, UP) => {
                    self.traverse_up = false;
                    Some((x - 1, y))
                }
                (7.., y, X_ODD, UP) => Some((x + 1, y - 1)),
                (7.., y, X_ODD, DOWN) => {
                    if y + 1 < size {
                        Some((x + 1, y + 1))
                    } else {
                        self.traverse_up = !self.traverse_up;
                        Some((x - 1, y))
                    }
                }
                _ => None,
            };
            self.next_position = next_pos;
            return Some((x, y));
        }
        None
    }
}

const SEG_MODE_BYTES: u8 = 0b0100;

// encode string to data code words
//include mode type and padding bits
pub fn encode_byte_segment(data: &str, out: &mut [u8]) -> Result<usize, EncodingErr> {
    let char_count = data.chars().count();
    if char_count > out.len() {
        return Err(DataTooLong);
    }
    let mut bit_writer = BigEndianBitWriter::new(out);
    //bytes
    bit_writer.append_bits(SEG_MODE_BYTES, 4);
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
    const PAD_BYTES: [u8; 2] = [0xEC, 0x11];
    for i in 0..bytes.len() {
        let b = PAD_BYTES[(i & 1)]; //cycle between odd and even
        bytes[i] = b;
    }
}
