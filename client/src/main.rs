use image::io::Reader;
use image::GenericImageView;

use std::io::Cursor;

fn main() {
    froggi::hello();

    let addr = if std::env::args().collect::<String>().contains("-l") {
        "127.0.0.1:11121"
    } else {
        include_str!("../server_address").trim()
    };

    println!("connecting to {}", addr);
    let result = froggi::send_request(addr, "index.fml").unwrap();
    println!("got {:?}", result);

    let _ = result.parse_page().unwrap();

    let reader = Reader::new(Cursor::new(result.items()[0].data()))
        .with_guessed_format()
        .expect("cursor never fails");
    let format = reader.format();
    match reader.decode() {
        Ok(image) => println!("got image: {:?} {:?}", format, image.dimensions()),
        Err(error) => println!("couldn't decode: {}", error),
    }
}
