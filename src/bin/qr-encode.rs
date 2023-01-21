use tiny_qr::bits::BitSquare;
use tiny_qr::codec::QrCode;
use tiny_qr::error_cc::ErrorLevel;
use tiny_qr::{codec, Canvas, BLACK, RED, WHITE};

const PIXEL_PER_MOD: u32 = 8;
const QUITE_ZONE_SIZE: u32 = 2;
const VERSION: u8 = 5;

fn main() {
    let module_sq_size: u32 = codec::version_to_size(VERSION) as u32;

    let mut qr = QrCode::new(VERSION, ErrorLevel::L);
    ppm_img(module_sq_size, qr.data_sq(), "qr-no-data");
    qr.encode_data(
        "Unless the Lord builds the house,
    the builders labor in vain. Psalm-127",
    );

    ppm_img(module_sq_size, qr.data_sq(), "qr-test");
    ppm_img(module_sq_size, qr.reserved_area(), "qr-test-reserved");
}

fn ppm_img(module_sq_size: u32, bit_sq: &BitSquare, filename: &str) {
    let canvas_size: u32 = ((module_sq_size + QUITE_ZONE_SIZE * 2) * PIXEL_PER_MOD) as u32;

    let mut picture = Canvas::new(canvas_size, canvas_size);
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
