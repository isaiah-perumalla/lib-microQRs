use crate::Version;
use std::fs::File;
use std::io::Write;

pub mod ppm {
    use crate::img::{Canvas, RED, RGB, WHITE};
    use crate::Code;
    use std::io::Write;

    pub fn to_img<const S: usize>(code: &Code<S>, colors: [RGB; 2], writer: &mut impl Write) {
        let mut img = Canvas::for_version(code.version);
        for m in code.version.reserved_iter() {
            let (x, y) = m.position();
            let i = usize::from(m.is_dark());
            img.set_pixel(x as u32, y as u32, &colors[i]);
        }
        for module in code.module_iter() {
            let i = usize::from(module.is_dark());
            let (x, y) = module.position();
            img.set_pixel(x as u32, y as u32, &colors[i]);
        }
        //write header
        writer
            .write_all(format!("P6 {} {} 255 ", img.width, img.height).as_bytes())
            .expect("error writing header");
        img.write(writer);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RGB(pub u8, pub u8, pub u8);

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
    const PIXEL_PER_MOD: u8 = 8;
    pub fn set_colour(&mut self, x: u32, y: u32, colour: &RGB) {
        if x > 0 && y > 0 && x < self.width && y < self.height {
            self.pixels[(self.width * y + x) as usize] = *colour;
        }
    }

    pub fn write_to_file(&mut self, filename: &str) {
        let mut file = init_ppm(filename, self.width, self.height);
        self.write(&mut file);
    }

    pub fn write_header(&self, writer: &mut impl Write) {
        writer
            .write_all(format!("P6 {} {} 255 ", self.width, self.height).as_bytes())
            .expect("error writing header");
    }
    pub fn write(&mut self, w: &mut impl Write) {
        let bytes = &serialize_rgb(&self.pixels, (self.width * self.height) as usize);
        w.write_all(bytes).expect("error");
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
            WHITE,
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

pub const WHITE: RGB = RGB(255, 255, 255);
pub const RED: RGB = RGB(255, 0, 0);

pub const GREY: RGB = RGB(169, 169, 169);
pub const GREEN: RGB = RGB(0, 255, 0);
pub const YELLOW: RGB = RGB(255, 0, 255);
pub const ORANGE: RGB = RGB(255, 165, 0);

pub const BLACK: RGB = RGB(0, 0, 0);
