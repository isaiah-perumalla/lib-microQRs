use tiny_qr::bits::BitSquare;
use tiny_qr::codec::QrCode;
use tiny_qr::error_cc::ErrorLevel;
use tiny_qr::{codec, Canvas, BLACK, RED, WHITE, Version, GREY, RGB, Module, GREEN};

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

    let mut picture = Canvas::new(canvas_size, canvas_size, GREY,
                                  QUITE_ZONE_SIZE as u8, PIXEL_PER_MOD as u8 );
    for y in 0..module_sq_size {
        for x in 0..module_sq_size {
            let color = if bit_sq.is_set(x as u8, y as u8) {
                RED
            } else {
                WHITE
            };
            picture.set_pixel( x, y, &color);
        }
    }
    picture.write_to_file(filename);
}

fn module_to_color(m: Module) -> RGB {
    if m.is_data() {
         return if m.is_dark() {BLACK} else { GREEN };
    }
    else {
        return if m.is_dark() {RED} else { WHITE };
    }
}
fn render_ppm_img(version:Version, filename: &str, color_fn: impl Fn(Module) -> RGB) {
    let square_size = version.square_size() as u32;
    let canvas_size: u32 = ((square_size + QUITE_ZONE_SIZE * 2) * PIXEL_PER_MOD) as u32;

    let mut picture = Canvas::new(canvas_size, canvas_size, GREY, QUITE_ZONE_SIZE as u8, PIXEL_PER_MOD as u8);
    for module in version.reserved_iter() {
        let (x,y) = module.position();
        let x = x as u32;
        let y = y as u32;
        let color = if module.is_dark() {RED} else { WHITE };
        picture.set_pixel( x, y, &color);
        }
    picture.write_to_file(filename);

}



#[cfg(test)]
mod tests {
    use tiny_qr::Version;
    use crate::{module_to_color, render_ppm_img};


    #[test] //visual test to see mandatory/reserved areas are rendered
    fn test_reserved_area() {
        render_ppm_img(Version(1), "version-1", module_to_color);
        render_ppm_img(Version(2), "version-2", module_to_color);
        render_ppm_img(Version(3), "version-3", module_to_color);
        render_ppm_img(Version(4), "version-4", module_to_color);
        render_ppm_img(Version(5), "version-5", module_to_color);
    }
}