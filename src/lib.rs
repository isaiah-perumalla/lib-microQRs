extern crate core;

use std::fs::File;
use std::io::Write;

pub mod bits;
pub mod codec;
pub mod error_cc;
pub mod gf256;

pub struct Version(u8);

impl Version {
    //alignment square position for version 1-5
    const ALIGNMENT_POSITIONS:[ &'static[(u8,u8)];6] = [&[], &[],
                                                &[(18,18)], &[(22,22)],
                                                &[(26,26)], &[(30,30)]];
    pub fn square_size(&self) -> u8 {
        4 * self.0 + 17
    }
    fn dark_module_pos(&self) -> (u8, u8) {
        (8, 4 * self.0 + 9)
    }

    fn alignment_squares_iter(&self) -> impl Iterator<Item=ConcentricSquare> {
        let mut i = 0;
        let square_positions = Self::ALIGNMENT_POSITIONS[self.0 as usize];
        std::iter::from_fn(move || {
            if i < square_positions.len() {
                let pos = square_positions[i];
                i += 1;
                Some(ConcentricSquare{center: pos, size: 3, color_bits: 0b101})
            }
            else {
                None
            }
        })
    }

    fn finding_pattern(&self) -> [ConcentricSquare; 3] {
        let size = self.square_size();
        [
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
        ]
    }

    pub(crate) fn reserved_iter(&self) -> impl Iterator<Item = (u8, u8, bool)> {
        let finding_pat = self.finding_pattern();
        let alignment_square_iter = self.alignment_squares_iter();
        std::iter::from_fn(move || {
            None
        })
    }
}

#[derive(Copy, Clone, Debug)]
struct ConcentricSquare {
    center: (u8, u8),
    size: u8,
    color_bits: u8,
}

impl ConcentricSquare {
    const EMPTY: ConcentricSquare = ConcentricSquare{center:(0,0), size:0 ,color_bits:0};
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

    pub fn new(width: u32, height: u32) -> Canvas {
        Canvas {
            width,
            height,
            pixels: vec![WHITE; (width * height) as usize],
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
        let squares:Vec<ConcentricSquare> = Version(2).alignment_squares_iter().collect();
        assert_eq!(squares.len(), 1);
        let square = squares.get(0).unwrap();

        let set:HashSet<(u8,u8,bool)> = HashSet::from_iter(square.iter_squares());
        assert_eq!(true, set.contains(&(17,17,false)));
        assert_eq!(true, set.contains(&(16,16,true)));
        assert_eq!(false, set.contains(&(15,15,true)));

    }

    #[test]
    fn test_version_reserved_area_iter() {
        let v = Version(1);
        let set:HashSet<(u8,u8,bool)> = HashSet::from_iter(v.reserved_iter());

    }
}
