use froggi::request::Request;
use froggi::response::{Item, ItemKind, Response, ResponseKind};

use std::io::Write;
use std::net::{TcpListener, TcpStream};

fn handle_client(mut stream: TcpStream) {
    let request = Request::from_bytes(&mut stream).unwrap();

    println!("request: {:?}", request);

    // todo: verify markup is correct
    // todo: some sort of page and page data cache
    let page = String::from(include_str!("../pages/test_markup.fml"));
    let header_img_data = include_bytes!("../pages/red_toy_small.png");
    let mut header_img = Vec::new();
    header_img.extend_from_slice(header_img_data);

    let response = Response::new(
        ResponseKind::Page,
        page,
        vec![Item::new(
            String::from("red_toy_small.png"),
            ItemKind::Png,
            header_img,
        )],
    );

    println!("client: {}", response.id());

    stream.write_all(&response.into_bytes()).unwrap();
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:11121").unwrap();
    println!("listening at {}", listener.local_addr().unwrap());
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("new client");
                std::thread::spawn(move || handle_client(stream));
            }
            Err(e) => {
                println!("error {}", e);
            }
        }
    }
}
