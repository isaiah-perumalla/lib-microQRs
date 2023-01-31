use std::env;
use std::io::stdout;
use microQRs::img::{BLACK, WHITE};

///
/// simple usage for qr library
/// Reads text from stdin and output a QR code in PPM format https://en.wikipedia.org/wiki/Netpbm
fn main() {
    let mut args = env::args();
    if let Some(data) = args.nth(1) {
        let result = microQRs::encode::<144>(&data);
        if let Ok(code) = result {
            microQRs::img::ppm::to_img(&code, [WHITE, BLACK], &mut stdout());
        } else {
            eprintln!("encode err ");
        }
    } else {
        println!("usage simple <text-to-encode> ");
    }
}
