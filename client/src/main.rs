use image::io::Reader;
use image::GenericImageView;

use std::io::Cursor;

fn main() {
    froggi::hello();
    println!("connecting");
    let result = if std::env::args().collect::<String>().contains("-l") {
        froggi::send_request("127.0.0.1:11121", "index.fml").unwrap()
    } else {
        froggi::send_request(include_str!("../server_address").trim(), "index.fml").unwrap()
    };
    println!("got {:?}", result);

    let reader = Reader::new(Cursor::new(result.items()[0].data()))
        .with_guessed_format()
        .expect("cursor never fails");
    let format = reader.format();
    match reader.decode() {
        Ok(image) => println!("got image: {:?} {:?}", format, image.dimensions()),
        Err(error) => println!("couldn't decode: {}", error),
    }
}
