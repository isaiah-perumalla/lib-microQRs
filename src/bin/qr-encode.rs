use tiny_qr::bits::BitSquare;
use tiny_qr::codec::QrCode;
use tiny_qr::error_cc::ErrorLevel;
use tiny_qr::{codec, Canvas, BLACK, RED, WHITE, Version, GREY};

const PIXEL_PER_MOD: u32 = 8;
const QUITE_ZONE_SIZE: u32 = 2;
const VERSION: u8 = 5;

fn main() {
    let module_sq_size: u32 = codec::version_to_size(VERSION) as u32;

    let mut qr = QrCode::new(VERSION, ErrorLevel::L);
    ppm_img(module_sq_size, qr.data_sq(), "qr-no-data");
    qr.encode_data(
        "Psalm-127 Unless the Lord builds the house,
    the builders labor in vain.",
    );

    ppm_img(module_sq_size, qr.data_sq(), "qr-test");
    ppm_img(module_sq_size, qr.reserved_area(), "qr-test-reserved");
}

fn ppm_img(module_sq_size: u32, bit_sq: &BitSquare, filename: &str) {
    let canvas_size: u32 = ((module_sq_size + QUITE_ZONE_SIZE * 2) * PIXEL_PER_MOD) as u32;

    let mut picture = Canvas::new(canvas_size, canvas_size, WHITE);
    for y in 0..module_sq_size {
        for x in 0..module_sq_size {
            let c = if bit_sq.is_set(x as u8, y as u8) {
                RED
            } else {
                WHITE
            };
            for i in 0..PIXEL_PER_MOD {
                for j in 0..PIXEL_PER_MOD {
                    picture.set_colour(
                        (x + QUITE_ZONE_SIZE) * PIXEL_PER_MOD + i,
                        (y + QUITE_ZONE_SIZE) * PIXEL_PER_MOD + j,
                        &c,
                    );
                }
            }
        }
    }
    picture.write_to_file(filename);
}


fn render_ppm_img(version:Version, filename: &str) {
    let square_size = version.square_size() as u32;
    let canvas_size: u32 = ((square_size + QUITE_ZONE_SIZE * 2) * PIXEL_PER_MOD) as u32;

    let mut picture = Canvas::new(canvas_size, canvas_size, GREY);
    for (x,y,is_dark) in version.reserved_iter() {
        let x = x as u32;
        let y = y as u32;
        let color = if is_dark {RED} else { WHITE };
            for i in 0..PIXEL_PER_MOD {
                for j in 0..PIXEL_PER_MOD {
                    picture.set_colour(
                        (x + QUITE_ZONE_SIZE) * PIXEL_PER_MOD + i,
                        (y + QUITE_ZONE_SIZE) * PIXEL_PER_MOD + j,
                        &color,
                    );
                }
            }
        }
    picture.write_to_file(filename);

}


#[cfg(test)]
mod tests {
    use tiny_qr::Version;
    use crate::render_ppm_img;


    #[test] //visual test to see mandatory/reserved areas are rendered
    fn test_reserved_area() {
        render_ppm_img(Version(1), "version-1");
        render_ppm_img(Version(2), "version-2");
        render_ppm_img(Version(3), "version-3");
        render_ppm_img(Version(4), "version-4");
        render_ppm_img(Version(5), "version-5");
    }
}