use tiny_qr::{Canvas, BLACK, WHITE};
use tiny_qr::bits::BitSquare;
use tiny_qr::qr::{ErrorLevel, QrCode};

const PIXEL_PER_MOD: u32 = 16;
const QUITE_ZONE_SIZE: u32 = 2;
const VERSION: u8 = 1;

const WORDS: [u8; 26] = [0x40, 0xD4, 0xA4, 0x55, 0x35, 0x55, 0x32, 0x06, 0x97, 0x32, 0x04, 0xB4,
    0x94, 0xE4, 0x70, 0xEC, 0x11, 0xEC, 0x11, 0xAD, 0xC9, 0x1E, 0xB1, 0x19, 0xCE, 0x38];

fn main() {
    let module_sq_size: u32 = tiny_qr::qr::version_to_size(VERSION) as u32;
    let code_words =  [0x41, 0x16, 0xA6, 0x56, 0x56, 0x36, 0x52, 0x06, 0x97, 0x32, 0x07, 0x46,
        0x86, 0x52, 0x06, 0x26, 0x57, 0x37, 0x40, 0x1B, 0xCA, 0xF0, 0x58, 0xB8, 0x03, 0x61];


    let mut qr = QrCode::new(VERSION, ErrorLevel::L);
    qr.set_code_words(&WORDS);
    qr.apply_mask(0);
    ppm_img(module_sq_size, qr.data_sq(), "qr-test");
    ppm_img(module_sq_size, qr.reserved_area(), "qr-test-reserved");
}

fn ppm_img(module_sq_size: u32, bit_sq: &BitSquare, filename: &str) {
    let canvas_size: u32 = ((module_sq_size + QUITE_ZONE_SIZE * 2) * PIXEL_PER_MOD) as u32;

    let mut picture = Canvas::new(canvas_size, canvas_size);
    for y in 0..module_sq_size {
        for x in 0..module_sq_size {
            let c = if bit_sq.is_set(x as u8, y as u8) { BLACK } else { WHITE };
            for i in 0..PIXEL_PER_MOD {
                for j in 0..PIXEL_PER_MOD {
                    picture.set_colour((x + QUITE_ZONE_SIZE) * PIXEL_PER_MOD + i,
                                       (y + QUITE_ZONE_SIZE) * PIXEL_PER_MOD + j, &c);
                }
            }
        }
    }
    picture.write_to_file(filename);
}
