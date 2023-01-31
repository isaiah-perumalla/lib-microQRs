use std::env;
use std::io::stdout;
use std::process::exit;
use microQRs::img::{BLACK, Canvas, RED, RGB, WHITE};
use microQRs::Version;

fn main() {

    let mut args = env::args();

    let arg = args.nth(1);
    if arg.is_none() {
        eprintln!("<version> required, specify version number; ");
        exit(1);
    }
    let version = match arg.unwrap().as_str() {
        "1" => Version(1),
        "2" => Version(2),
        "3" => Version(3),
        "4" => Version(4),
        "5" => Version(5),
        x => {
            eprintln!("invalid/unsupported version {}", x);
            exit(1);
        }
    };

            let mut ppm_img = Canvas::for_version(version);
            let res = version.reserved_iter().fold(&mut ppm_img, |acc, module| {
                const MOD_COLOR: [RGB; 2] = [WHITE, BLACK];
                let (x, y) = module.position();
                let mod_color = usize::from(module.is_dark());
                acc.set_pixel(x as u32, y as u32, &MOD_COLOR[mod_color % 2]);
                acc
            });

            let mut color_iter = rgb_iter()
                                .map(|c| std::iter::repeat(c).take(8))
                                .flat_map(|f| f);
            version
                .data_region_iter().zip(color_iter)
                .fold(&mut ppm_img, |acc, ((x,y), c)| {
                    acc.set_pixel(x as u32, y as u32, &c);
                    acc
                });
            ppm_img.write_header(&mut stdout());
            ppm_img.write(&mut stdout());
}



fn rgb_iter() -> impl Iterator<Item=RGB> {
    const COLOR_PALLETE: [[u8;3];12] = [
        [88, 51, 60], [23, 97, 38], [239, 205, 178], [137, 162, 61],
            [31, 31, 252], [206, 162, 125], [87, 156, 198], [61, 209, 255],
                             [51, 65, 98], [49, 30, 168], [255, 99, 117], [217, 33, 33]];
    let size = COLOR_PALLETE.len();
    let to_rgb = |arr:[u8;3]| RGB(arr[0],arr[1], arr[2]);
    let mut i = 0;
    std::iter::from_fn(move|| {
        let next_color = to_rgb(COLOR_PALLETE[i % size]);
        i += 1;
        Some(next_color)
    })
}