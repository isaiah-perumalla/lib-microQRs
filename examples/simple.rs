fn main() {
    let data = "Unless the Lord builds the house,
    the builders labor in vain. Psalm-127 www.biblegateway.com/passage";
    let result = tiny_qr::encode::<144>(data);
    if let Ok(code) = result {
        println!("code_words={:?}", code.code_words());
        // to_ppm_img(&code, "qr-code");
    } else {
        eprintln!("encode err ");
    }
}
