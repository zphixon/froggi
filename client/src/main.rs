use image::io::Reader;
use image::GenericImageView;

use froggi::request::RequestKind;

use std::io::Cursor;

use image::ImageEncoder;

fn main() {
    let local = std::env::args().collect::<String>().contains("-l");
    //let server = include_str!("../server_address").trim();
    let server = "gang-and-friends.com:11121";

    let addr = if local { "127.0.0.1:11121" } else { server };

    println!(
        "connecting to {}",
        if local { addr } else { "a secret server" }
    );

    let result = froggi::send_request(addr, "test_markup.fml", RequestKind::Page).unwrap();

    println!("got {:#?}", result);
    match result.parse() {
        Ok(page) => {
            println!("page ok: {:#?}", page);
        }

        Err(errors) => {
            for error in errors {
                println!("{}", error);
            }
        }
    }

    if result.items().len() > 0 {
        let reader = Reader::new(Cursor::new(result.items()[0].data()))
            .with_guessed_format()
            .expect("cursor never fails");
        let format = reader.format();
        match reader.decode() {
            Ok(image) => println!("got image: {:?} {:?}", format, image.dimensions()),
            Err(error) => println!("couldn't decode: {}", error),
        }
    }
}
