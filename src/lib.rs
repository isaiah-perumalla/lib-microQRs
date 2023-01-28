extern crate core;

use crate::bits::MsbBitIter;
use crate::codec::encoder::{add_padding, encode_byte_segment};
use crate::codec::EncodingErr::DataTooLong;
use crate::codec::{EncodingErr, MaskFN, ZigzagIter, MASK_FN};
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
mod tests {
    use crate::codec::{QrCode, ZigzagIter};
    use crate::error_cc::ErrorLevel;
    use crate::{encode, ConcentricSquare, Module, Version};
    use std::collections::HashSet;

    #[test]
    fn test_concentric_square_iter() {
        let sq = ConcentricSquare {
            center: (3, 3),
            size: 4,
            color_bits: 0b1011,
        };
        let set: HashSet<(u8, u8, bool)> = HashSet::from_iter(sq.iter_squares());
        assert_eq!(set.len(), 49);
        check_contains_module_square(&set, 7u8, (0, 0), true);

        let sq = ConcentricSquare {
            center: (17, 3),
            size: 4,
            color_bits: 0b1011,
        };
        let set: HashSet<(u8, u8, bool)> = HashSet::from_iter(sq.iter_squares());
        assert_eq!(set.len(), 49);

        check_contains_module_square(&set, 7u8, (14, 0), true);
        check_contains_module_square(&set, 5u8, (15, 1), false);
        check_contains_module_square(&set, 3u8, (16, 2), true);
        check_contains_module_square(&set, 1u8, (17, 3), true);

        let sq = ConcentricSquare {
            center: (17, 3),
            size: 3,
            color_bits: 0b1011,
        };
        let set: HashSet<(u8, u8, bool)> = HashSet::from_iter(sq.iter_squares());
        println!("{:?}", &set);
        assert_eq!(set.len(), 25);

        check_contains_module_square(&set, 5u8, (15, 1), false);
    }

    fn check_contains_module_square(
        set: &HashSet<(u8, u8, bool)>,
        size: u8,
        top_left: (u8, u8),
        is_black: bool,
    ) {
        let (top_left_x, top_left_y) = top_left;
        for i in 0..size {
            assert_eq!(
                set.contains(&(i + top_left_x, top_left_y, is_black)),
                true,
                "does not contain ({},{}, {})",
                i + top_left_x,
                top_left_y,
                is_black
            );
            assert_eq!(
                set.contains(&(i + top_left_x, top_left_y + size - 1, is_black)),
                true,
                "does not contain ({},{},{})",
                i + top_left_x,
                top_left_y + size - 1,
                is_black
            );
            assert_eq!(
                set.contains(&(top_left_x, i + top_left_y, is_black)),
                true,
                "does not contain ({},{},{})",
                top_left_x,
                i + top_left_y,
                is_black
            );
            assert_eq!(
                set.contains(&(top_left_x + size - 1, i + top_left_y, is_black)),
                true,
                "does not contain ({},{},{})",
                top_left_x + size - 1,
                i + top_left_y,
                is_black
            );
        }
    }

    #[test]
    fn test_alignment_square() {
        assert_eq!(Version(1).alignment_squares_iter().count(), 0);
        let squares: Vec<ConcentricSquare> = Version(2).alignment_squares_iter().collect();
        assert_eq!(squares.len(), 1);
        let square = squares.get(0).unwrap();

        let set: HashSet<(u8, u8, bool)> = HashSet::from_iter(square.iter_squares());
        assert_eq!(true, set.contains(&(17, 17, false)));
        assert_eq!(true, set.contains(&(16, 16, true)));
        assert_eq!(false, set.contains(&(15, 15, true)));
    }

    #[test]
    fn test_version_reserved_area_iter() {
        for v_num in 1..=5 {
            let v = Version(v_num);

            let size = v.square_size();
            let check_reserved_sq = |top_left, bottom_right| {
                let (top_left_x, top_left_y) = top_left;
                let (bottom_right_x, bottom_right_y) = bottom_right;
                for x in top_left_x..=bottom_right_x {
                    for y in top_left_y..=bottom_right_y {
                        assert_eq!(
                            false,
                            v.is_data_location((x, y)),
                            "data module location {},{} version={}",
                            x,
                            y,
                            v_num
                        );
                    }
                }
            };

            check_reserved_sq((0, 0), (8, 8));
            check_reserved_sq((size - 8, 0), (size - 1, 8));
            check_reserved_sq((0, size - 8), (8, size - 1));
            assert_eq!(true, v.is_data_location((size - 9, 0)));
            assert_eq!(false, v.is_data_location(v.dark_module_location()));
            assert_eq!(false, v.is_data_location((6, 0)));
            assert_eq!(false, v.is_data_location((6, 9)));

            if v_num == 2 {
                //check alignment pattern
                check_reserved_sq((16, 16), (20, 20));
                assert_eq!(
                    true,
                    v.is_data_location((15, 15)),
                    "should be data areas (15,15)"
                );
                assert_eq!(
                    true,
                    v.is_data_location((21, 21)),
                    "should be data areas (15,15)"
                );
            }
            if v_num == 3 {
                //check alignment pattern
                check_reserved_sq((20, 20), (24, 24));
            }
            if v_num == 4 {
                //check alignment pattern
                check_reserved_sq((24, 24), (28, 28));
            }
            if v_num == 5 {
                //check alignment pattern version 5
                check_reserved_sq((28, 28), (32, 32));
            }
        }
    }

    #[test]
    pub fn test_data_region_iter() {
        const VERSION: u8 = 2;
        let qr = QrCode::new(VERSION, ErrorLevel::L);
        let data_modules: Vec<(u8, u8)> = ZigzagIter::new(Version(VERSION).square_size())
            .filter(|p| qr.is_data_module(*p))
            .collect();

        println!("{:?}", &data_modules);
        let mods: Vec<(u8, u8)> = Version(VERSION).data_region_iter().collect();
        let expected_order = &[
            (24u8, 24u8),
            (23, 24),
            (24, 23),
            (23, 23),
            (24, 22),
            (23, 22),
            (24, 21),
            (23, 21),
            (24, 20),
            (23, 20),
            (24, 19),
            (23, 19),
            (24, 18),
            (23, 18),
            (24, 17),
            (23, 17),
            (24, 16),
            (23, 16),
            (24, 15),
            (23, 15),
            (24, 14),
            (23, 14),
            (24, 13),
            (23, 13),
            (24, 12),
            (23, 12),
            (24, 11),
            (23, 11),
            (24, 10),
            (23, 10),
            (24, 9),
            (23, 9),
            (22, 9),
            (21, 9),
            (22, 10),
            (21, 10),
            (22, 11),
            (21, 11),
            (22, 12),
            (21, 12),
            (22, 13),
            (21, 13),
            (22, 14),
            (21, 14),
            (22, 15),
            (21, 15),
            (22, 16),
            (21, 16),
            (22, 17),
            (21, 17),
            (22, 18),
            (21, 18),
            (22, 19),
            (21, 19),
            (22, 20),
            (21, 20),
            (22, 21),
            (21, 21),
            (22, 22),
            (21, 22),
            (22, 23),
            (21, 23),
            (22, 24),
            (21, 24),
            (20, 24),
            (19, 24),
            (20, 23),
            (19, 23),
            (20, 22),
            (19, 22),
            (20, 21),
            (19, 21),
            (20, 15),
            (19, 15),
            (20, 14),
            (19, 14),
            (20, 13),
            (19, 13),
            (20, 12),
            (19, 12),
            (20, 11),
            (19, 11),
            (20, 10),
            (19, 10),
            (20, 9),
            (19, 9),
            (18, 9),
            (17, 9),
            (18, 10),
            (17, 10),
            (18, 11),
            (17, 11),
            (18, 12),
            (17, 12),
            (18, 13),
            (17, 13),
            (18, 14),
            (17, 14),
            (18, 15),
            (17, 15),
            (18, 21),
            (17, 21),
            (18, 22),
            (17, 22),
            (18, 23),
            (17, 23),
            (18, 24),
            (17, 24),
            (16, 24),
            (15, 24),
            (16, 23),
            (15, 23),
            (16, 22),
            (15, 22),
            (16, 21),
            (15, 21),
            (15, 20),
            (15, 19),
            (15, 18),
            (15, 17),
            (15, 16),
            (16, 15),
            (15, 15),
            (16, 14),
            (15, 14),
            (16, 13),
            (15, 13),
            (16, 12),
            (15, 12),
            (16, 11),
            (15, 11),
            (16, 10),
            (15, 10),
            (16, 9),
            (15, 9),
            (16, 8),
            (15, 8),
            (16, 7),
            (15, 7),
            (16, 5),
            (15, 5),
            (16, 4),
            (15, 4),
            (16, 3),
            (15, 3),
            (16, 2),
            (15, 2),
            (16, 1),
            (15, 1),
            (16, 0),
            (15, 0),
            (14, 0),
            (13, 0),
            (14, 1),
            (13, 1),
            (14, 2),
            (13, 2),
            (14, 3),
            (13, 3),
            (14, 4),
            (13, 4),
            (14, 5),
            (13, 5),
            (14, 7),
            (13, 7),
            (14, 8),
            (13, 8),
            (14, 9),
            (13, 9),
            (14, 10),
            (13, 10),
            (14, 11),
            (13, 11),
            (14, 12),
            (13, 12),
            (14, 13),
            (13, 13),
            (14, 14),
            (13, 14),
            (14, 15),
            (13, 15),
            (14, 16),
            (13, 16),
            (14, 17),
            (13, 17),
            (14, 18),
            (13, 18),
            (14, 19),
            (13, 19),
            (14, 20),
            (13, 20),
            (14, 21),
            (13, 21),
            (14, 22),
            (13, 22),
            (14, 23),
            (13, 23),
            (14, 24),
            (13, 24),
            (12, 24),
            (11, 24),
            (12, 23),
            (11, 23),
            (12, 22),
            (11, 22),
            (12, 21),
            (11, 21),
            (12, 20),
            (11, 20),
            (12, 19),
            (11, 19),
            (12, 18),
            (11, 18),
            (12, 17),
            (11, 17),
            (12, 16),
            (11, 16),
            (12, 15),
            (11, 15),
            (12, 14),
            (11, 14),
            (12, 13),
            (11, 13),
            (12, 12),
            (11, 12),
            (12, 11),
            (11, 11),
            (12, 10),
            (11, 10),
            (12, 9),
            (11, 9),
            (12, 8),
            (11, 8),
            (12, 7),
            (11, 7),
            (12, 5),
            (11, 5),
            (12, 4),
            (11, 4),
            (12, 3),
            (11, 3),
            (12, 2),
            (11, 2),
            (12, 1),
            (11, 1),
            (12, 0),
            (11, 0),
            (10, 0),
            (9, 0),
            (10, 1),
            (9, 1),
            (10, 2),
            (9, 2),
            (10, 3),
            (9, 3),
            (10, 4),
            (9, 4),
            (10, 5),
            (9, 5),
            (10, 7),
            (9, 7),
            (10, 8),
            (9, 8),
            (10, 9),
            (9, 9),
            (10, 10),
            (9, 10),
            (10, 11),
            (9, 11),
            (10, 12),
            (9, 12),
            (10, 13),
            (9, 13),
            (10, 14),
            (9, 14),
            (10, 15),
            (9, 15),
            (10, 16),
            (9, 16),
            (10, 17),
            (9, 17),
            (10, 18),
            (9, 18),
            (10, 19),
            (9, 19),
            (10, 20),
            (9, 20),
            (10, 21),
            (9, 21),
            (10, 22),
            (9, 22),
            (10, 23),
            (9, 23),
            (10, 24),
            (9, 24),
            (8, 16),
            (7, 16),
            (8, 15),
            (7, 15),
            (8, 14),
            (7, 14),
            (8, 13),
            (7, 13),
            (8, 12),
            (7, 12),
            (8, 11),
            (7, 11),
            (8, 10),
            (7, 10),
            (8, 9),
            (7, 9),
            (5, 9),
            (4, 9),
            (5, 10),
            (4, 10),
            (5, 11),
            (4, 11),
            (5, 12),
            (4, 12),
            (5, 13),
            (4, 13),
            (5, 14),
            (4, 14),
            (5, 15),
            (4, 15),
            (5, 16),
            (4, 16),
            (3, 16),
            (2, 16),
            (3, 15),
            (2, 15),
            (3, 14),
            (2, 14),
            (3, 13),
            (2, 13),
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
            (1, 13),
            (0, 13),
            (1, 14),
            (0, 14),
            (1, 15),
            (0, 15),
            (1, 16),
            (0, 16),
        ];
        assert_eq!(&mods, &expected_order);
    }

    #[test]
    pub fn test_encode() {
        let code = encode::<128>("isaiah-perumalla").unwrap();

        //ErrorLevel::L
        let expected_words = [
            65, 6, 151, 54, 22, 150, 22, 130, 215, 6, 87, 39, 86, 214, 22, 198, 198, 16, 236, 121,
            93, 100, 23, 7, 230, 143,
        ];
        assert_eq!(&expected_words, code.code_words());
    }

    #[test]
    pub fn test_version_format_modules() {
        let v = Version(1);
        let modules = v.format_modules(ErrorLevel::L, 0);
        assert_eq!(true, modules.iter().all(|m| !m.is_data()));

        println!("{:?}", &modules);
    }
}
