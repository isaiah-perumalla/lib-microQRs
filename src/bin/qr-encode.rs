use tiny_qr::bits::BitSquare;
use tiny_qr::codec::{EncodingErr, QrCode};
use tiny_qr::error_cc::ErrorLevel;
use tiny_qr::{codec, Canvas, Module, Version, BLACK, GREEN, GREY, RED, RGB, WHITE, Code, YELLOW, ORANGE};

const PIXEL_PER_MOD: u32 = 8;
const QUITE_ZONE_SIZE: u32 = 2;
const VERSION: u8 = 5;

fn main() {
    let module_sq_size: u32 = codec::version_to_size(VERSION) as u32;

    let mut qr = QrCode::new(VERSION, ErrorLevel::L);
    ppm_img(module_sq_size, qr.data_sq(), "qr-no-data");
    let data = "Psalm-127 Unless the Lord builds the house,
    the builders labor in vain.";
    qr.encode_data(data);

    ppm_img(module_sq_size, qr.data_sq(), "qr-test");
    ppm_img(module_sq_size, qr.reserved_area(), "qr-test-reserved");

    let code   = tiny_qr::encode::<128>(data);
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
    use crate::{module_to_color};
    use tiny_qr::{BLACK, Canvas, GREEN, ORANGE, RED, RGB, Version, WHITE, YELLOW};

    #[test] //visual test to see mandatory/reserved areas are rendered
    fn test_reserved_area() {
        let version = Version(1);
        let mut ppm_img = Canvas::for_version(version);
        let res = version.reserved_iter().fold(&mut ppm_img, |acc, module| {
            const MOD_COLOR: [RGB;2] = [RED, WHITE];
            let (x,y) = module.position();
            let mod_color = usize::from(module.is_dark());
            acc.set_pixel(x as u32 ,y as u32, &MOD_COLOR[mod_color%2]);
            acc
        });
        version.data_region_iter().enumerate().fold(&mut ppm_img,  |acc, (i, pos)| {
            const DATA_COLORS:[RGB;4] = [GREEN, YELLOW, BLACK, ORANGE];
            let color = DATA_COLORS[(i>>3)%4]; //give each group of byte sized bits a unique color
            acc.set_pixel(pos.0 as u32, pos.1 as u32, &color);
            acc
        } );
        ppm_img.write_to_file("version-1");
        // render_ppm_img(Version(1), "version-1", module_to_color);
        // render_ppm_img(Version(2), "version-2", module_to_color);
        // render_ppm_img(Version(3), "version-3", module_to_color);
        // render_ppm_img(Version(4), "version-4", module_to_color);
        // render_ppm_img(Version(5), "version-5", module_to_color);
    }


}
