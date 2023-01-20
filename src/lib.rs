extern crate core;

use std::fs::File;
use std::io::Write;

pub mod bits;
pub mod error_cc;
pub mod gf256;
pub mod codec;

pub const WHITE: RGB = RGB {
    red: 255,
    green: 255,
    blue: 255,
};
pub const GREY: RGB = RGB {
    red: 169,
    green: 169,
    blue: 169,
};
pub const BLACK: RGB = RGB {
    red: 0,
    green: 0,
    blue: 0,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RGB {
    red: u8,
    green: u8,
    blue: u8,
}

fn serialize_rgb(pixels: &Vec<RGB>, size: usize) -> Vec<u8> {
    let mut output: Vec<u8> = Vec::with_capacity(size * 3);
    for pix in pixels {
        output.push(pix.red);
        output.push(pix.green);
        output.push(pix.blue);
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
