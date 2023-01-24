extern crate core;

use std::fs::File;
use std::io::{Read, Write};

pub mod bits;
pub mod codec;
pub mod error_cc;
pub mod gf256;

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

pub struct Version(pub u8);

impl Version {
    //alignment square position for version 1-5
    const ALIGNMENT_POSITIONS: [&'static [(u8, u8)]; 6] =
        [&[], &[], &[(18, 18)], &[(22, 22)], &[(26, 26)], &[(30, 30)]];
    pub fn square_size(&self) -> u8 {
        4 * self.0 + 17
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
        let mut squares = [
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
        let (top_left_x, top_left_y) = (self.center.0 - size - 1, self.center.1 - size - 1);
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
        /* slow
        for pixel in &self.pixels {
            file.write_all(&[pixel.red, pixel.green, pixel.blue]).expect("error writing to a file");
        }*/
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
    use crate::{ConcentricSquare, Version};
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
}
