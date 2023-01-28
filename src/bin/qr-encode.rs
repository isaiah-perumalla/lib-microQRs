use tiny_qr::bits::BitSquare;
use tiny_qr::error_cc::ErrorLevel;
use tiny_qr::{BLACK, Canvas, Code, codec, EncodingErr, GREEN, GREY, Module, ORANGE, RED, RGB, Version, WHITE, YELLOW};

const PIXEL_PER_MOD: u32 = 16;
const QUITE_ZONE_SIZE: u32 = 2;
const VERSION: u8 = 5;

fn main() {

    let data = "Unless the Lord builds the house,
    the builders labor in vain. Psalm-127 www.biblegateway.com/passage";


    let result = tiny_qr::encode::<144>(data);
    if let Ok(code) = result {
        to_ppm_img(&code, "qr-code");
    } else {
        eprintln!("encode err ");
    }
}

fn to_ppm_img<const S: usize>(code: &Code<S>, file_name: &str) {
    let mut img = Canvas::for_version(code.version);
    const COLORS: [RGB; 2] = [WHITE, RED];
    for m in code.version.reserved_iter() {
        let (x, y) = m.position();
        let i = usize::from(m.is_dark());
        img.set_pixel(x as u32, y as u32, &COLORS[i]);
    }
    for module in code.module_iter() {
        let i = usize::from(module.is_dark());
        let (x, y) = module.position();
        img.set_pixel(x as u32, y as u32, &COLORS[i]);
    }

    img.write_to_file(file_name);
}

fn ppm_img(module_sq_size: u32, bit_sq: &BitSquare, filename: &str) {
    let canvas_size: u32 = ((module_sq_size + QUITE_ZONE_SIZE * 2) * PIXEL_PER_MOD) as u32;

    let mut picture = Canvas::new(
        canvas_size,
        canvas_size,
        GREY,
        QUITE_ZONE_SIZE as u8,
        PIXEL_PER_MOD as u8,
    );
    for y in 0..module_sq_size {
        for x in 0..module_sq_size {
            let color = if bit_sq.is_set(x as u8, y as u8) {
                RED
            } else {
                WHITE
            };
            picture.set_pixel(x, y, &color);
        }
    }
    picture.write_to_file(filename);
}

fn module_to_color(m: Module) -> RGB {
    if m.is_data() {
        return if m.is_dark() { BLACK } else { GREEN };
    } else {
        return if m.is_dark() { RED } else { WHITE };
    }
}

#[cfg(test)]
mod tests {
    use crate::module_to_color;
    use tiny_qr::{BLACK, Canvas, GREEN, ORANGE, RED, RGB, Version, WHITE, YELLOW};

    #[test] //visual lib to see mandatory/reserved areas are rendered
    fn test_reserved_area() {
        for i in 1..=5 {
            let version = Version(i);
            let mut ppm_img = Canvas::for_version(version);
            let res = version.reserved_iter().fold(&mut ppm_img, |acc, module| {
                const MOD_COLOR: [RGB; 2] = [WHITE, RED];
                let (x, y) = module.position();
                let mod_color = usize::from(module.is_dark());
                acc.set_pixel(x as u32, y as u32, &MOD_COLOR[mod_color % 2]);
                acc
            });
            version
                .data_region_iter()
                .enumerate()
                .fold(&mut ppm_img, |acc, (i, pos)| {
                    const DATA_COLORS: [RGB; 4] = [GREEN, YELLOW, BLACK, ORANGE];
                    let color = DATA_COLORS[(i >> 3) % 4]; //give each group of byte sized bits a unique color
                    acc.set_pixel(pos.0 as u32, pos.1 as u32, &color);
                    acc
                });
            ppm_img.write_to_file(&format!("version-{i}"));
        }
    }
}
