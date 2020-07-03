use image::io::Reader;
use image::GenericImageView;

use std::fs::File;
use std::io::{Cursor, Write};

fn main() {
    froggi::hello();

    let addr = if std::env::args().collect::<String>().contains("-l") {
        "127.0.0.1:11121"
    } else {
        include_str!("../server_address").trim()
    };

    println!("connecting to {}", addr);
    let result = froggi::send_request(addr, "new_test_markup.fml").unwrap();

    println!("got {:#?}", result);
    match result.parse() {
        Ok(doc) => {
            println!("page ok: {:#?}", doc);

            let html = froggi::markup::to_html(&doc);
            println!("{}", html);

            let mut file = File::create("server/pages/new_generated_test_markup.html").unwrap();
            file.write_all(html.as_bytes()).unwrap();
        }

        Err(errors) => {
            for error in errors {
                println!("{}", error);
            }
        }
    }

    let reader = Reader::new(Cursor::new(result.items()[0].data()))
        .with_guessed_format()
        .expect("cursor never fails");
    let format = reader.format();
    match reader.decode() {
        Ok(image) => println!("got image: {:?} {:?}", format, image.dimensions()),
        Err(error) => println!("couldn't decode: {}", error),
    }
}
